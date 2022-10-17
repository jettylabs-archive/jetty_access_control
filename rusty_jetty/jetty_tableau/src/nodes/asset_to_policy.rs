use jetty_core::connectors::nodes as jetty_nodes;

use crate::coordinator::Environment;

use super::{
    Datasource, Flow, FromTableau, IntoTableau, Lens, Metric, Project, TableauCualable, View,
    Workbook,
};

/// We can't create a generic impl for a trait defined outside the current
/// crate, so instead this macro makes it quick to define the impls for
/// each individual asset.
macro_rules! impl_from_asset_to_policy {
    ($struct:ty) => {
        impl FromTableau<$struct> for Vec<jetty_nodes::Policy> {
            fn from(val: $struct, env: &Environment) -> Self {
                let cual = val.cual(env).uri();
                val.permissions
                    .into_iter()
                    .map(|p| {
                        let mut policy: jetty_nodes::Policy = Into::into(p.clone());
                        policy.governs_assets.insert(cual.to_owned());
                        policy
                    })
                    .collect()
            }
        }
    };
}

impl_from_asset_to_policy!(Flow);
impl_from_asset_to_policy!(Workbook);
impl_from_asset_to_policy!(Lens);
impl_from_asset_to_policy!(Project);
impl_from_asset_to_policy!(Datasource);
impl_from_asset_to_policy!(Metric);
impl_from_asset_to_policy!(View);

/// Given an asset from Tableau, bundle up its permissions
/// as Jetty policies.
pub(crate) fn env_to_jetty_policies<T>(
    asset: &mut dyn Iterator<Item = T>,
    env: &Environment,
) -> Vec<jetty_nodes::Policy>
where
    T: IntoTableau<Vec<jetty_nodes::Policy>>,
    Vec<jetty_core::connectors::nodes::Policy>: FromTableau<T>,
{
    asset
        .flat_map(|f| -> Vec<jetty_nodes::Policy> { f.into(env) })
        .collect()
}
