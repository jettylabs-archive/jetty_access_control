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
        impl FromTableau<$struct> for Vec<jetty_nodes::RawPolicy> {
            fn from(val: $struct, env: &Environment) -> Self {
                let cual = val.cual(env).uri();
                val.permissions
                    .into_iter()
                    .map(|p| {
                        let mut policy: jetty_nodes::RawPolicy = Into::into(p.clone());
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
fn asset_to_jetty_policies<T>(
    asset: &mut dyn Iterator<Item = T>,
    env: &Environment,
) -> Vec<jetty_nodes::RawPolicy>
where
    T: IntoTableau<Vec<jetty_nodes::RawPolicy>>,
    Vec<jetty_core::connectors::nodes::RawPolicy>: FromTableau<T>,
{
    asset
        .flat_map(|f| -> Vec<jetty_nodes::RawPolicy> { f.into(env) })
        .collect()
}

pub(crate) fn env_to_jetty_policies(env: &Environment) -> Vec<jetty_nodes::RawPolicy> {
    let flow_policies: Vec<jetty_nodes::RawPolicy> =
        asset_to_jetty_policies(&mut env.flows.clone().into_values(), env);
    let project_policies: Vec<jetty_nodes::RawPolicy> =
        asset_to_jetty_policies(&mut env.projects.clone().into_values(), env);
    let lens_policies: Vec<jetty_nodes::RawPolicy> =
        asset_to_jetty_policies(&mut env.lenses.clone().into_values(), env);
    let datasource_policies: Vec<jetty_nodes::RawPolicy> =
        asset_to_jetty_policies(&mut env.datasources.clone().into_values(), env);
    let workbook_policies: Vec<jetty_nodes::RawPolicy> =
        asset_to_jetty_policies(&mut env.workbooks.clone().into_values(), env);
    let metric_policies: Vec<jetty_nodes::RawPolicy> =
        asset_to_jetty_policies(&mut env.metrics.clone().into_values(), env);
    let view_policies: Vec<jetty_nodes::RawPolicy> =
        asset_to_jetty_policies(&mut env.views.clone().into_values(), env);
    flow_policies
        .into_iter()
        .chain(project_policies.into_iter())
        .chain(lens_policies.into_iter())
        .chain(datasource_policies.into_iter())
        .chain(workbook_policies.into_iter())
        .chain(metric_policies.into_iter())
        .chain(view_policies.into_iter())
        .collect()
}
