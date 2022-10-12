
use std::collections::{HashMap, HashSet};

use crate::{GrantOf, Role, RoleName};

/// Create a map of role indices to the indices of their direct parents in the 
/// role vec.
pub(crate) fn create_role_tree<'a>(
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