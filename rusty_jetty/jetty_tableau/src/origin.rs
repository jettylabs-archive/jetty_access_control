use jetty_core::cual::Cual;

use serde::{Deserialize, Serialize};

use crate::{coordinator::Environment, nodes::TableauCualable, rest::TableauAssetType};

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Deserialize, Serialize)]
pub(crate) enum SourceOrigin {
    /// Another source other than Tableau. We just use the CUAL to identify it.
    Other { cual: Cual },
    /// A tableau source. All we need is the type and ID to identify it.
    Tableau {
        asset_type: TableauAssetType,
        id: String,
    },
}

impl SourceOrigin {
    pub(crate) fn into_cual(self, env: &Environment) -> Cual {
        match self {
            SourceOrigin::Other { cual } => cual,
            SourceOrigin::Tableau { asset_type, id } => match asset_type {
                TableauAssetType::Project => {
                    let asset = env.projects.get(&id).expect("getting asset from env");
                    asset.cual(env)
                }
                TableauAssetType::Datasource => {
                    let asset = env.datasources.get(&id).expect("getting asset from env");
                    asset.cual(env)
                }
                TableauAssetType::Flow => {
                    let asset = env.flows.get(&id).expect("getting asset from env");
                    asset.cual(env)
                }
                TableauAssetType::Workbook => {
                    let asset = env.workbooks.get(&id).expect("getting asset from env");
                    asset.cual(env)
                }
                TableauAssetType::Lens => {
                    let asset = env.lenses.get(&id).expect("getting asset from env");
                    asset.cual(env)
                }
                TableauAssetType::Metric => {
                    let asset = env.metrics.get(&id).expect("getting asset from env");
                    asset.cual(env)
                }
                TableauAssetType::View => {
                    let asset = env.views.get(&id).expect("getting asset from env");
                    asset.cual(env)
                }
            },
        }
    }
}
