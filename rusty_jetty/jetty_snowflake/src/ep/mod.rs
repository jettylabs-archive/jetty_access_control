//! Effective Permissions
//!

use std::collections::{HashMap, HashSet};

use jetty_core::{
    connectors::{
        nodes::{EffectivePermission, SparseMatrix, PermissionMode},
        UserIdentifier,
    },
    cual::Cual,
};

use crate::{coordinator::{Environment, Grantee}, GrantOf, Object, Role, RoleName, User, Grant};

struct EffectivePermissionMap<'a> {
    matrix: SparseMatrix<UserIdentifier, Cual, HashSet<EffectivePermission>>,
    roles: &'a Vec<Role>,
    role_map: HashMap<usize, HashSet<usize>>,
    role_grants: &'a HashMap<Grantee, HashSet<RoleName>>,
}

impl<'a> EffectivePermissionMap<'a> {
    pub(crate) fn new(env: &'a Environment, role_grants: &'a HashMap<Grantee, HashSet<RoleName>>) -> Self {
        Self {
            matrix: HashMap::new(),
            roles: &env.roles,
            role_map: create_role_map(&env.roles, &env.role_grants),
            role_grants,
        }
    }

    fn get_recursive_roles(&'a self, user:&User) -> HashSet<RoleName>{
        // Get the direct grants first.
        let direct_grants = self.role_grants.get(&Grantee::User(user.name.to_owned())).cloned().unwrap_or_default();
        let mut res =  HashSet::new();
        // Get recursive grants for each of the direct ones.
        res
    }

    fn get_recursive_roles_for_role(&self, RoleName(role):&RoleName) -> HashSet<RoleName>{
        let mut res = HashSet::new();
        // Get the direct grants for this role.
        let direct_grants = self.role_grants.get(&Grantee::Role(role.to_owned())).cloned().unwrap_or_default();
        // Get the recursive parents for each parent role.
        for role in &direct_grants{
            res.extend(self.get_recursive_roles_for_role(&role).into_iter());
        }
        res.extend(direct_grants.into_iter());
        res
    }

    fn get_effective_permissions_for_object<'b>(
        &self,
        env: &'b Environment,
        user: &'b User,
        object: &'b Object,
    ) -> impl Iterator<Item = EffectivePermission > + 'b {
        let user_roles = self.get_recursive_roles(user);
        // Get the db + schema permissions for this object.
        let db_grants:Vec<_>= env.standard_grants.iter().filter(|sg| {
            // Find grants of this db to any of the user's roles.
            sg.granted_on_name() == object.database_name && user_roles.contains(&RoleName(sg.role_name().to_owned()))
        }).collect();
        let schema_grants:Vec<_>= env.standard_grants.iter().filter(|sg| {
            // Find grants of this schema any of the user's roles.
            sg.granted_on_name() == object.schema_fqn() && user_roles.contains(&RoleName(sg.role_name().to_owned()))
        }).collect();

        // 2. Check whether this user has USAGE on db + schema.
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
        let has_schema_usage = schema_grants.iter().find(|g| g.privilege == "USAGE").is_some();

        if !has_any_db_grant || !has_schema_usage{
            // Deny access. Early return.
        }

        // resolve conflicts and return all effective privileges.
        let object_grants:HashMap<_,EffectivePermission> = env.standard_grants.iter().filter(|sg|{
            sg.granted_on_name() == object.fqn() && user_roles.contains(&RoleName(sg.role_name().to_owned()))
        }).fold(HashMap::new(),|mut map, grant|{
            if let Some(entry) = map.get(&grant.privilege){
                match entry.privilege{
                // resolve conflicts
                _ => ()
                };
            }else{
                map.insert(&grant.privilege, EffectivePermission::new(grant.privilege.to_owned(), PermissionMode::Allow, vec!["Privilege explicitly granted.".to_owned()]));
            }
            map
        });

        object_grants.into_values()
    }
}


/// Create a map of role indices to the indices of their direct parents in the 
/// role vec.
fn create_role_map<'a>(
    roles: &'a Vec<Role>,
    role_grants: &'a Vec<GrantOf>,
) -> HashMap<usize, HashSet<usize>> {
    // create a name->index map to use while we make the full index->index map.
    let name_index_map: HashMap<&String, usize> = roles
        .iter()
        .enumerate()
        .map(
            |(
                i,
                Role {
                    name: RoleName(role_name),
                },
            )| 
            // Flip the order so we can find index by role name.
            (role_name, i),
        )
        .collect();

    roles
        .iter()
        .enumerate()
        .fold(HashMap::new(), |mut map, (i, role)| {
            let parent_indices = get_direct_parents(role, role_grants)
                .flat_map(|RoleName(parent_name)| {
                    name_index_map.get(&parent_name).cloned()
                })
                .collect();
            map.insert(i, parent_indices);
            map
        })
}

/// Get all direct parents of the given role.
fn get_direct_parents<'a>(
    role: &'a Role,
    all_grants: &'a Vec<GrantOf>,
) -> impl Iterator<Item = &'a RoleName> {
    all_grants.iter().filter_map(|rg| {
        if rg.role == role.name {
            Some(&rg.role)
        } else {
            None
        }
    })
}

pub(crate) fn get_effective_permissions(
    env: &Environment,
) -> SparseMatrix<UserIdentifier, Cual, HashSet<EffectivePermission>> {
    let mut res = HashMap::new();
    res
}

