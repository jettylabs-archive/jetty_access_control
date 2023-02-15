//! Get lineage for each asset type and update that lineage on the env object.

use super::IdField;
use crate::{
    coordinator::{Coordinator, HasSources},
    origin::SourceOrigin,
};
use anyhow::Result;
use jetty_core::cual::Cual;
use jetty_core::logging::warn;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

pub(crate) struct AssetReferences {
    luid: String,
    upstream_table_ids: Vec<Cual>,
    downstream_table_ids: Vec<Cual>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssetReferencesResponse {
    name: String,
    luid: String,
    upstream_tables: Vec<IdField>,
    #[serde(default)]
    downstream_tables: Vec<IdField>,
}

/// fetches references (table ids) for different asset types
macro_rules! impl_fetch_references {
    ($t:tt, $b:tt) => {
        impl Coordinator {
            pub(super) async fn $b(
                &self,
                cual_map: &HashMap<String, Cual>,
                unsupported_sql: &HashSet<String>,
            ) -> Result<Vec<AssetReferences>> {
                let query = format!(
                    "query Assets {{
                {} {{
                    name
                    luid
                    upstreamTables {{
                      id
                    }}{}
                  }}
                }}",
                    stringify!($t),
                    if ["publishedDatasources", "flows"].contains(&stringify!($t)) {
                        "
                        downstreamTables {
                          id
                        }"
                    } else {
                        ""
                    }
                );
                let response: Vec<AssetReferencesResponse> = self
                    .graphql_query_to_object_vec(query.as_str(), vec!["data", stringify!($t)])
                    .await?;

                Ok(response
                    .into_iter()
                    .map(|r| {
                        for table in r.upstream_tables.iter().chain(r.downstream_tables.iter()) {
                            if unsupported_sql.contains(&table.id) {
                                warn!(
                                    "{} contains unsupported SQL: lineage may be incomplete",
                                    r.name
                                )
                            }
                        }

                        AssetReferences {
                            luid: r.luid,
                            upstream_table_ids: r
                                .upstream_tables
                                .into_iter()
                                .filter_map(|t| cual_map.get(&t.id).cloned())
                                .collect(),
                            downstream_table_ids: r
                                .downstream_tables
                                .into_iter()
                                .filter_map(|t| cual_map.get(&t.id).cloned())
                                .collect(),
                        }
                    })
                    .collect())
            }
        }
    };
}

impl_fetch_references!(workbooks, fetch_workbooks_references);
impl_fetch_references!(metrics, fetch_metrics_references);
impl_fetch_references!(flows, fetch_flows_references);
impl_fetch_references!(publishedDatasources, fetch_datasources_references);
impl_fetch_references!(lenses, fetch_lenses_references);

pub(super) fn update_sources<T: HasSources>(
    references: Vec<AssetReferences>,
    assets: &mut HashMap<String, T>,
) {
    for reference in references {
        if let Some(asset) = assets.get_mut(&reference.luid) {
            asset.set_sources((
                reference
                    .upstream_table_ids
                    .into_iter()
                    .map(|c| SourceOrigin::Other { cual: c })
                    .collect(),
                reference
                    .downstream_table_ids
                    .into_iter()
                    .map(|c| SourceOrigin::Other { cual: c })
                    .collect(),
            ))
        }
    }
}
