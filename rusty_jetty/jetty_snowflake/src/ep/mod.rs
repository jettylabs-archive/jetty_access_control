//! Effective Permissions
//!

use std::collections::{HashMap, HashSet};

use jetty_core::{
    connectors::{
        nodes::{EffectivePermission, SparseMatrix},
        UserIdentifier,
    },
    cual::Cual,
};

use crate::{coordinator::Environment, GrantOf, Object, Role, RoleName, User, Grant};

struct EffectivePermissionMap<'a> {
    matrix: SparseMatrix<UserIdentifier, Cual, HashSet<EffectivePermission>>,
    roles: &'a Vec<Role>,
    role_map: HashMap<usize, HashSet<usize>>,
    role_grants: &'a Vec<GrantOf>,
}

impl<'a> EffectivePermissionMap<'a> {
    pub(crate) fn new(env: &'a Environment) -> Self {
        Self {
            matrix: HashMap::new(),
            roles: &env.roles,
            role_map: create_role_map(&env.roles, &env.role_grants),
            role_grants: &&env.role_grants,
        }
    }

fn get_recursive_roles(&'a self, user:&User) -> impl Iterator<Item=&'a RoleName>{
    let mut res = vec![];
    let direct_roles :Vec<_>= self.role_grants.filter(|rg| rg.granted_to == GrantedTo::User && rg.granted_name == user.name).collect();
    res
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

fn get_effective_permissions_for_object(
    env: &Environment,
    user: &User,
    object: &Object,
) -> impl Iterator<Item = EffectivePermission> {
    // 1. Get the db + schema for this object.
    let database = env
        .databases
        .iter()
        .find(|d| d.name == object.database_name);
    let schema = env
        .schemas
        .iter()
        .find(|s| s.name == object.schema_name && s.database_name == object.database_name);

    // 2. Check whether this user has USAGE on db + schema.
    let all_standard_grants = env
        .standard_grants
        .iter()
        .filter(|g| g.name == object.fqn());
    // Some notes: "If any database privilege is granted to a role, that
    // role can take SQL actions on objects in a schema using fully-qualified
    // names. The role must have the USAGE privilege on the schema as well as
    // the required privilege or privileges on the object. To make a database
    // the active database in a user session, the USAGE privilege on the database
    // is required." (https://tinyurl.com/snow-db-exceptions)
    //
    // So the user should have USAGE on the schema and ANY permission on the
    // database in order to use this object.


    // 2. Climb from the user through all their roles to find and collect all
    // permissions between them.

    // 3. Compare to make sure the right permission is the effective one.

    vec![].into_iter()
}

fn has_db_permission(user:&User, db:&str){
    // how do we tell whether a user has been granted anything to a db? We need the recursive roles first.
    todo!()
}
