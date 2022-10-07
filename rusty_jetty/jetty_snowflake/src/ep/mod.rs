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

use crate::{coordinator::Environment, Object, Role, User};

struct EffectivePermissionMap<'a> {
    matrix: SparseMatrix<UserIdentifier, Cual, HashSet<EffectivePermission>>,
    roles: &'a Vec<Role>,
    role_map: HashMap<String, HashSet<usize>>,
}

impl<'a> EffectivePermissionMap<'a> {
    pub(crate) fn new(env: &'a Environment) -> Self {
        Self {
            matrix: HashMap::new(),
            roles: &env.roles,
            role_map: create_role_map(&env.roles),
        }
    }
}

fn create_role_map(roles: &Vec<Role>) -> HashMap<String, HashSet<usize>> {
    todo!()
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

    // 2. Check for USAGE on db + schema.
    // let grants = env.standard_grants.filter(|g| g.name == object.fqn());

    // 2. Climb from the user through all their roles to find all permissions between them.
    // 3. Compare to make sure the right permission is the effective one.

    vec![].into_iter()
}
