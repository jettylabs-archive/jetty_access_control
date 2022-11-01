use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Context;
use futures::future::join_all;
use futures::future::BoxFuture;
use futures::StreamExt;

use jetty_core::connectors::nodes;
use jetty_core::connectors::AssetType;
use jetty_core::connectors::UserIdentifier;

use jetty_core::connectors::nodes::EffectivePermission;
use jetty_core::connectors::nodes::SparseMatrix;
use jetty_core::cual::Cualable;
use jetty_core::logging::debug;
use jetty_core::logging::error;
use jetty_core::permissions::matrix::InsertOrMerge;

use super::cual::{self, cual, get_cual_account_name, Cual};
use crate::consts::DATABASE;
use crate::consts::SCHEMA;
use crate::consts::TABLE;
use crate::consts::VIEW;
use crate::efperm::EffectivePermissionMap;
use crate::entry_types;
use crate::entry_types::ObjectKind;
use crate::entry_types::RoleName;
use crate::Asset;
use crate::Grant;
use crate::GrantType;

/// Number of metadata request to run currently (e.g. permissions).
/// ~20 seems to give the best performance. In some circumstances, we may want to bump this up.
const CONCURRENT_METADATA_FETCHES: usize = 20;

/// Environment is a collection of objects pulled right out of Snowflake.
/// We process them to make jetty nodes and edges.
#[derive(Default, Debug)]
pub(crate) struct Environment {
    pub(crate) databases: Vec<entry_types::Database>,
    pub(crate) schemas: Vec<entry_types::Schema>,
    pub(crate) objects: Vec<entry_types::Object>,
    pub(crate) users: Vec<entry_types::User>,
    pub(crate) roles: Vec<entry_types::Role>,
    pub(crate) standard_grants: Vec<entry_types::StandardGrant>,
    pub(crate) future_grants: Vec<entry_types::FutureGrant>,
    pub(crate) role_grants: Vec<entry_types::GrantOf>,
}

// Now lets start filling up the environment

pub(super) struct Coordinator<'a> {
    pub(crate) env: Environment,
    conn: &'a super::SnowflakeConnector,
    pub(crate) role_grants: HashMap<Grantee, HashSet<RoleName>>,
}

#[derive(Hash, Eq, PartialEq)]
pub(crate) enum Grantee {
    User(String),
    Role(String),
}

impl<'a> Coordinator<'a> {
    pub(super) fn new(conn: &'a super::SnowflakeConnector) -> Self {
        Self {
            env: Default::default(),
            role_grants: Default::default(),
            conn,
        }
    }

    pub(super) async fn get_data(&mut self) -> nodes::ConnectorData {
        // // Run in one group
        // Get all databases
        // Get all the schemas
        // Get all the users
        // Get all the roles

        let hold: Vec<BoxFuture<_>> = vec![
            Box::pin(self.conn.get_databases_future(&mut self.env.databases)),
            Box::pin(self.conn.get_schemas_future(&mut self.env.schemas)),
            Box::pin(self.conn.get_users_future(&mut self.env.users)),
            Box::pin(self.conn.get_roles_future(&mut self.env.roles)),
        ];

        let results = join_all(hold).await;
        for res in results {
            if let Err(e) = res {
                error!("{}", e)
            }
        }

        // try one object:
        let mut hold: Vec<BoxFuture<_>> = vec![];

        // for each schema, get objects
        let objects_mutex = Arc::new(Mutex::new(&mut self.env.objects));
        for schema in &self.env.schemas {
            let m = Arc::clone(&objects_mutex);
            hold.push(Box::pin(self.conn.get_objects_futures(schema, m)));
        }

        // for each role, get grants to that role
        let grants_to_role_mutex = Arc::new(Mutex::new(&mut self.env.standard_grants));
        for role in &self.env.roles {
            let m = Arc::clone(&grants_to_role_mutex);
            hold.push(Box::pin(self.conn.get_grants_to_role_future(role, m)));
        }

        // for each role, get grants of
        let target_arc = Arc::new(Mutex::new(&mut self.env.role_grants));
        for role in &self.env.roles {
            let m = Arc::clone(&target_arc);
            hold.push(Box::pin(self.conn.get_grants_of_role_future(role, m)));
        }

        // for each schema, get future grants
        let future_grants_arc = Arc::new(Mutex::new(&mut self.env.future_grants));
        for schema in &self.env.schemas {
            let m = Arc::clone(&future_grants_arc);
            hold.push(Box::pin(
                self.conn.get_future_grants_of_schema_future(schema, m),
            ));
        }

        // for database, get future grants, using the same Arc<Mutex>
        for database in &self.env.databases {
            let m = Arc::clone(&future_grants_arc);
            hold.push(Box::pin(
                self.conn.get_future_grants_of_database_future(database, m),
            ));
        }

        let results = futures::stream::iter(hold)
            .buffer_unordered(CONCURRENT_METADATA_FETCHES)
            .collect::<Vec<_>>()
            .await;

        for res in results {
            if let Err(e) = res {
                error!("{}", e)
            }
        }

        self.role_grants = self.build_role_grants();

        nodes::ConnectorData {
            // 19 Sec
            groups: self.get_jetty_groups(),
            // 7 Sec
            users: self.get_jetty_users(),
            // 3.5 Sec
            assets: self.get_jetty_assets(),
            tags: self.get_jetty_tags(),
            policies: self.get_jetty_policies(),
            effective_permissions: self.get_effective_permissions(),
            asset_references: Default::default(),
            cual_prefix: Some(
                cual::get_cual_prefix()
                    .context("cual account not yet set")
                    .unwrap(),
            ),
        }
    }

    /// Get the role grants into a nicer format
    fn build_role_grants(&self) -> HashMap<Grantee, HashSet<RoleName>> {
        let mut res: HashMap<Grantee, HashSet<RoleName>> = HashMap::new();
        for grant in &self.env.role_grants {
            let key = match &grant.granted_to[..] {
                "ROLE" => Grantee::Role(grant.grantee_name.to_owned()),
                "USER" => Grantee::User(grant.grantee_name.to_owned()),
                other => {
                    debug!("skipping unexpected role type: {}", other);
                    continue;
                }
            };

            if let Some(v) = res.get_mut(&key) {
                v.insert(grant.role.to_owned());
            } else {
                res.insert(key, HashSet::from([grant.role.to_owned()]));
            }
        }
        res
    }

    /// Get standard grants grants by roles
    /// Snowflake doesn't allow permissions to be granted to users
    fn get_standard_grants_by_role(&self) -> HashMap<String, Vec<GrantType>> {
        let mut res: HashMap<String, Vec<GrantType>> = HashMap::new();
        for grant in &self.env.standard_grants {
            if let Some(v) = res.get_mut(grant.role_name()) {
                v.push(GrantType::Standard(grant.to_owned()));
            } else {
                res.insert(
                    grant.role_name().to_owned(),
                    vec![GrantType::Standard(grant.to_owned())],
                );
            }
        }
        res
    }

    /// Get future grants grants by roles
    /// Snowflake doesn't allow permissions to be granted to users
    fn get_future_grants_by_role(&self) -> HashMap<String, Vec<GrantType>> {
        let mut res: HashMap<String, Vec<GrantType>> = HashMap::new();
        for grant in &self.env.future_grants {
            if let Some(v) = res.get_mut(grant.role_name()) {
                v.push(GrantType::Future(grant.to_owned()));
            } else {
                res.insert(
                    grant.role_name().to_owned(),
                    vec![GrantType::Future(grant.to_owned())],
                );
            }
        }
        res
    }

    /// Helper fn to get role grants for a grantee
    fn get_role_grant_names(&self, grantee: &Grantee) -> HashSet<String> {
        if let Some(g) = self.role_grants.get(grantee) {
            g.iter()
                .map(|r| {
                    let RoleName(role_name) = r;
                    role_name.to_owned()
                })
                .collect()
        } else {
            HashSet::new()
        }
    }

    /// Get groups from environment
    fn get_jetty_groups(&self) -> Vec<nodes::RawGroup> {
        let mut res = vec![];
        for role in &self.env.roles {
            let RoleName(role_name) = &role.name;
            res.push(nodes::RawGroup::new(
                role_name.to_owned(),
                HashMap::new(),
                self.get_role_grant_names(&Grantee::Role(role_name.to_owned())),
                HashSet::new(),
                HashSet::new(),
                HashSet::new(),
            ))
        }
        res
    }

    /// Get users from environment
    fn get_jetty_users(&self) -> Vec<nodes::RawUser> {
        let mut res = vec![];
        for user in &self.env.users {
            // only add user identifiers if they are not blank
            let mut identifiers = HashSet::new();
            if !user.email.is_empty() {
                identifiers.insert(UserIdentifier::Email(user.email.to_owned()));
            };
            if !user.first_name.is_empty() {
                identifiers.insert(UserIdentifier::FirstName(user.first_name.to_owned()));
            };
            if !user.last_name.is_empty() {
                identifiers.insert(UserIdentifier::LastName(user.last_name.to_owned()));
            };
            if !user.first_name.is_empty() && !user.last_name.is_empty() {
                identifiers.insert(UserIdentifier::FullName(format!(
                    "{} {}",
                    user.first_name, user.last_name
                )));
            };
            if !user.display_name.is_empty() {
                identifiers.insert(UserIdentifier::Other(user.display_name.to_owned()));
            };
            if !user.login_name.is_empty() {
                identifiers.insert(UserIdentifier::Other(user.login_name.to_owned()));
            };

            res.push(nodes::RawUser::new(
                user.name.to_owned(),
                identifiers,
                HashMap::new(),
                self.get_role_grant_names(&Grantee::User(user.name.to_owned())),
                HashSet::new(),
            ))
        }
        res
    }

    /// get assets from environment
    fn get_jetty_assets(&self) -> Vec<nodes::RawAsset> {
        let mut res = vec![];
        for object in &self.env.objects {
            let object_type = match object.kind {
                ObjectKind::Table => TABLE,
                ObjectKind::View => VIEW,
            };

            res.push(nodes::RawAsset::new(
                object.cual(),
                "".to_owned(),
                AssetType(object_type.to_owned()),
                HashMap::new(),
                // Policies applied are handled in get_jetty_policies
                HashSet::new(),
                HashSet::from([cual!(object.database_name, object.schema_name).uri()]),
                // Handled in child_of for parents.
                HashSet::new(),
                // We aren't extracting lineage from Snowflake right now.
                HashSet::new(),
                HashSet::new(),
                HashSet::new(),
            ));
        }

        for schema in &self.env.schemas {
            res.push(nodes::RawAsset::new(
                schema.cual(),
                format!("{}.{}", schema.database_name, schema.name),
                AssetType(SCHEMA.to_owned()),
                HashMap::new(),
                // Policies applied are handled in get_jetty_policies
                HashSet::new(),
                HashSet::from([cual!(schema.database_name).uri()]),
                // Handled in child_of for parents.
                HashSet::new(),
                // We aren't extracting lineage from Snowflake right now.
                HashSet::new(),
                HashSet::new(),
                HashSet::new(),
            ));
        }

        for db in &self.env.databases {
            res.push(nodes::RawAsset::new(
                db.cual(),
                db.name.to_owned(),
                AssetType(DATABASE.to_owned()),
                HashMap::new(),
                // Policies applied are handled in get_jetty_policies
                HashSet::new(),
                HashSet::new(),
                // Handled in child_of for parents.
                HashSet::new(),
                // We aren't extracting lineage from Snowflake right now.
                HashSet::new(),
                HashSet::new(),
                HashSet::new(),
            ));
        }

        res
    }

    /// get tags from environment
    /// NOT CURRENTLY IMPLEMENTED - This is an enterprise-only feature
    fn get_jetty_tags(&self) -> Vec<nodes::RawTag> {
        vec![]
    }

    /// get policies from environment
    fn get_jetty_policies(&self) -> Vec<nodes::RawPolicy> {
        let mut res = vec![];

        // For standard grants
        for (_role, grants) in self.get_standard_grants_by_role() {
            res.extend(self.conn.grants_to_policies(&grants))
        }

        // For future grants
        for (_role, grants) in self.get_future_grants_by_role() {
            res.extend(self.conn.grants_to_policies(&grants))
        }

        res
    }

    /// get effective_permissions from environment
    pub(crate) fn get_effective_permissions(
        &self,
    ) -> SparseMatrix<String, Cual, HashSet<EffectivePermission>> {
        let mut res = HashMap::new();
        let ep_map = EffectivePermissionMap::new(&self.role_grants);

        // The runtime performance here can definitely be improved, but this is
        // a workable naive approach for now.
        for user in &self.env.users {
            let mut obj_eps = HashMap::new();
            for obj in &self.env.objects {
                obj_eps.insert_or_merge(
                    obj.cual(),
                    ep_map.get_effective_permissions_for_object(&self.env, user, obj),
                );
            }
            res.insert_or_merge(user.name.to_owned(), obj_eps);
            let mut db_eps = HashMap::new();
            for db in &self.env.databases {
                db_eps.insert_or_merge(
                    db.cual(),
                    ep_map.get_effective_permissions_for_asset(
                        &self.env,
                        user,
                        &Asset::Database(db.clone()),
                    ),
                );
            }
            res.insert_or_merge(user.name.to_owned(), db_eps);
            let mut schema_eps = HashMap::new();
            for schema in &self.env.schemas {
                schema_eps.insert_or_merge(
                    schema.cual(),
                    ep_map.get_effective_permissions_for_asset(
                        &self.env,
                        user,
                        &Asset::Schema(schema.clone()),
                    ),
                )
            }
            res.insert_or_merge(user.name.to_owned(), schema_eps);
        }

        res
    }
}
