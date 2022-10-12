use std::collections::{HashMap, HashSet};

use jetty_core::{
    connectors::{
        nodes::{EffectivePermission, PermissionMode, SparseMatrix},
        UserIdentifier,
    },
    cual::Cual,
};

use crate::{
    coordinator::{Environment, Grantee},
    entry_types::ObjectKind,
    Asset, Grant, Object, Role, RoleName, User,
};

use super::{
    privilege::{TABLE_PRIVILEGES, VIEW_PRIVILEGES},
    role_tree::create_role_tree,
};

pub(crate) struct EffectivePermissionMap<'a> {
    matrix: SparseMatrix<UserIdentifier, Cual, HashSet<EffectivePermission>>,
    roles: &'a Vec<Role>,
    role_map: HashMap<usize, HashSet<usize>>,
    role_grants: &'a HashMap<Grantee, HashSet<RoleName>>,
}

impl<'a> EffectivePermissionMap<'a> {
    pub(crate) fn new(
        env: &'a Environment,
        role_grants: &'a HashMap<Grantee, HashSet<RoleName>>,
    ) -> Self {
        Self {
            matrix: HashMap::new(),
            roles: &env.roles,
            role_map: create_role_tree(&env.roles, &env.role_grants),
            role_grants,
        }
    }

    fn get_recursive_roles(&'a self, user: &User) -> HashSet<RoleName> {
        // Get the direct grants first.
        let direct_grants = self
            .role_grants
            .get(&Grantee::User(user.name.to_owned()))
            .cloned()
            .unwrap_or_default();
        let mut res = HashSet::new();
        // Get recursive grants for each of the direct ones.
        res
    }

    fn get_recursive_roles_for_role(&self, RoleName(role): &RoleName) -> HashSet<RoleName> {
        let mut res = HashSet::new();
        // Get the direct grants for this role.
        let direct_grants = self
            .role_grants
            .get(&Grantee::Role(role.to_owned()))
            .cloned()
            .unwrap_or_default();
        // Get the recursive parents for each parent role.
        for role in &direct_grants {
            res.extend(self.get_recursive_roles_for_role(&role).into_iter());
        }
        res.extend(direct_grants.into_iter());
        res
    }

    pub(crate) fn get_effective_permissions_for_asset(
        &self,
        env: &Environment,
        user: &User,
        asset: &Asset,
    ) -> HashSet<EffectivePermission> {
        if let Asset::Object(object) = asset {
            self.get_effective_permissions_for_object(env, user, object)
        } else {
            let user_roles = self.get_recursive_roles(user);
            self.get_granted_permissions(asset, &user_roles, env)
        }
    }

    pub(crate) fn get_effective_permissions_for_object<'b>(
        &self,
        env: &'b Environment,
        user: &'b User,
        object: &'b Object,
    ) -> HashSet<EffectivePermission> {
        let user_roles = self.get_recursive_roles(user);
        // Get the db + schema permissions for this object.
        let db_grants: Vec<_> = env
            .standard_grants
            .iter()
            .filter(|sg| {
                // Find grants of this db to any of the user's roles.
                sg.granted_on_name() == object.database_name
                    && user_roles.contains(&RoleName(sg.role_name().to_owned()))
            })
            .collect();
        let schema_grants: Vec<_> = env
            .standard_grants
            .iter()
            .filter(|sg| {
                // Find grants of this schema any of the user's roles.
                sg.granted_on_name() == object.schema_fqn()
                    && user_roles.contains(&RoleName(sg.role_name().to_owned()))
            })
            .collect();

        // Check whether the user is disabled right now.
        if user.disabled {
            return get_effective_permissions_for_all_privileges(
                object.kind,
                PermissionMode::Deny,
                vec!["User is disabled".to_owned()],
            );
        }
        // Check whether this user has USAGE on db + schema.
        // Some notes: "If any database privilege is granted to a role, that
        // role can take SQL actions on objects in a schema using fully-qualified
        // names. The role must have the USAGE privilege on the schema as well as
        // the required privilege or privileges on the object. To make a database
        // the active database in a user session, the USAGE privilege on the database
        // is required." (https://tinyurl.com/snow-db-exceptions)
        //
        // So the user should have USAGE on the schema and ANY permission on the
        // database in order to use this object.
        let has_any_db_grant = !db_grants.is_empty();
        let has_schema_usage = schema_grants
            .iter()
            .find(|g| g.privilege == "USAGE")
            .is_some();

        if !has_any_db_grant || !has_schema_usage {
            return get_effective_permissions_for_all_privileges(
                object.kind,
                PermissionMode::Deny,
                vec!["User does not have usage on the parent db and schema.".to_owned()],
            );
        }

        // Return all effective privileges.
        self.get_granted_permissions(&Asset::Object(object.clone()), &user_roles, env)
    }

    fn get_granted_permissions(
        &self,
        asset: &Asset,
        roles: &HashSet<RoleName>,
        env: &Environment,
    ) -> HashSet<EffectivePermission> {
        let object_grants: HashMap<_, EffectivePermission> = env
            .standard_grants
            .iter()
            .filter(|sg| {
                sg.granted_on_name() == asset.fqn()
                    && roles.contains(&RoleName(sg.role_name().to_owned()))
            })
            .fold(HashMap::new(), |mut map, grant| {
                // Insert the privilege if it doesn't already exist.
                if map.get_mut(&grant.privilege).is_none() {
                    map.insert(
                        &grant.privilege,
                        EffectivePermission::new(
                            grant.privilege.to_owned(),
                            PermissionMode::Allow,
                            vec!["Privilege explicitly granted.".to_owned()],
                        ),
                    );
                }
                map
            });

        object_grants.into_values().collect()
    }
}

fn get_effective_permissions_for_all_privileges(
    kind: ObjectKind,
    mode: PermissionMode,
    reasons: Vec<String>,
) -> HashSet<EffectivePermission> {
    let privileges = match kind {
        crate::entry_types::ObjectKind::Table => TABLE_PRIVILEGES,
        crate::entry_types::ObjectKind::View => VIEW_PRIVILEGES,
    };
    privileges
        .into_iter()
        .map(|&p| EffectivePermission::new(p.to_owned(), mode.clone(), reasons.clone()))
        .collect()
}
