use jetty_core::{
    connectors::nodes as jetty_nodes,
    cual::{Cual, Cualable},
};

use super::{Datasource, Flow, Lens, Metric, Project, View, Workbook};

macro_rules! impl_from_asset_to_policy {
    ($struct:ty) => {
        impl From<$struct> for Vec<jetty_nodes::Policy> {
            fn from(val: $struct) -> Self {
                val.permissions
                    .into_iter()
                    .map(|p| {
                        let mut policy: jetty_nodes::Policy = p.into();
                        policy.governs_assets.insert(val.cual.uri());
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

pub(crate) fn env_to_jetty_policies(
    asset: &mut dyn Iterator<Item = impl Into<Vec<jetty_nodes::Policy>>>,
) -> Vec<jetty_nodes::Policy> {
    asset
        .map(|f| -> Vec<jetty_nodes::Policy> { f.into() })
        .flatten()
        .collect()
}
