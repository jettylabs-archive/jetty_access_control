use anyhow::{anyhow, bail, Context, Result};
use petgraph::stable_graph::NodeIndex;

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Display;

use yaml_peg::{parse, repr::RcRepr, NodeRc};

use crate::access_graph::{AccessGraph, AssetAttributes, JettyNode, NodeName};
use crate::connectors::processed_nodes::ProcessedTag;
use crate::jetty::ConnectorNamespace;

use super::parser_common::{get_optional_bool, get_optional_string, indicated_msg};

/// The configuration of a tag
#[derive(Debug)]
pub(crate) struct TagConfig {
    description: Option<String>,
    value: Option<String>,
    pass_through_hierarchy: bool,
    pass_through_lineage: bool,
    apply_to: Option<Vec<TargetAsset>>,
    remove_from: Option<Vec<TargetAsset>>,
    _pos: u64,
}

/// information collected when unable to find a referenced asset
struct AssetMatchError {
    target: TargetAsset,
    problem: AssetMatchProblem,
    pos: u64,
}

impl Display for AssetMatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.problem {
            AssetMatchProblem::TooMany(v) => write!(
                f,
                "unable to disambiguate asset:\n{}\ncould refer to any of the following:\n{}",
                self.target,
                v.iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
            AssetMatchProblem::BadType(v) => write!(
                f,
                "unable to find asset with:\n{}\ndid you mean one of the following:\n{}",
                self.target,
                v.iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
            AssetMatchProblem::NoMatches => {
                write!(f, "unable to find asset:\n{}", self.target)
            }
        }
    }
}

/// Why we were unable to find a referenced asset. `TooMan` and `BadType` variants include a list of alternatives
enum AssetMatchProblem {
    TooMany(Vec<TargetAsset>),
    BadType(Vec<TargetAsset>),
    NoMatches,
}

/// The description of an asset a tag is pointing to
#[derive(Debug, Clone)]
pub(crate) struct TargetAsset {
    name: String,
    asset_type: Option<String>,
    pos: u64,
}

impl From<AssetAttributes> for TargetAsset {
    fn from(a: AssetAttributes) -> Self {
        TargetAsset {
            name: a.name().name_for_string_matching(),
            asset_type: Some(a.asset_type().to_string()),
            pos: 0,
        }
    }
}

impl Display for TargetAsset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(t) = &self.asset_type {
            write!(f, " - name: {}\n   asset_type: {t}", self.name)
        } else {
            write!(f, " - name: {}", self.name)
        }
    }
}

/// Parse a string tag config into a hashmap of tag configuration objects
pub(crate) fn parse_tags(config: &String) -> Result<HashMap<String, TagConfig>> {
    // Parse into Node representation and selecting the first element in the Vec
    let root = &parse::<RcRepr>(config).context("unable to parse tags file - invalid yaml")?[0];

    // Return an empty HashMap if there are no tags
    if root.is_null() {
        return Ok(Default::default());
    }
    let mapped_root = root
        .as_map()
        .map_err(|_| anyhow!("improperly formatted tags file: should be a dictionary of tag names and configurations \
        - please refer to the documentation for more information"))?;

    let mut tags = HashMap::new();

    // iterate over each tag
    for (k, v) in mapped_root {
        let tag_pos = k.pos();
        let tag_name = k
            .as_str()
            .map_err(|_| {
                anyhow!(
                    "tag name not a string: {}",
                    indicated_msg(config.as_bytes(), k.pos(), 2)
                )
            })?
            .to_owned();

        // get the (optional) description
        let description = get_optional_string(v.clone(), "description", config)?;

        // get the (optional) value
        let value = get_optional_string(v.clone(), "value", config)?;

        // get the (optional) pass_through_lineage flag
        let pass_through_lineage =
            if let Some(v) = get_optional_bool(v.clone(), "pass_through_lineage", config)? {
                v
            } else {
                false
            };

        let pass_through_hierarchy =
            if let Some(v) = get_optional_bool(v.clone(), "pass_through_hierarchy", config)? {
                v
            } else {
                false
            };

        let apply_to = v
            .get("apply_to")
            .ok()
            .map(|a| parse_target_assets(a.clone(), config))
            .transpose()?;

        let remove_from = v
            .get("remove_from")
            .ok()
            .map(|a| parse_target_assets(a.clone(), config))
            .transpose()?;

        tags.insert(
            tag_name,
            TagConfig {
                description,
                value,
                apply_to,
                remove_from,
                _pos: tag_pos,
                pass_through_hierarchy,
                pass_through_lineage,
            },
        );
    }

    Ok(tags)
}

fn parse_target_assets(node: NodeRc, config: &String) -> Result<Vec<TargetAsset>> {
    let asset_list = node.as_seq().map_err(|_| {
        anyhow!(
            "assets should be a list: {}",
            indicated_msg(config.as_bytes(), node.pos(), 2)
        )
    })?;

    let mut target_assets = vec![];

    for asset in asset_list {
        let target_asset = match asset.yaml() {
            yaml_peg::Yaml::Str(val) => TargetAsset {
                name: val.to_owned(),
                asset_type: None,
                pos: asset.pos(),
            },
            yaml_peg::Yaml::Map(_) => TargetAsset {
                name: get_optional_string(asset.clone(), "name", config)?.ok_or_else(|| {
                    anyhow!(
                        "asset name can't be blank: {}",
                        indicated_msg(config.as_bytes(), node.pos(), 2)
                    )
                })?,
                asset_type: get_optional_string(asset.clone(), "type", config)?,
                pos: asset.pos(),
            },
            _ => bail!(
                "unable to parse asset: {}",
                indicated_msg(config.as_bytes(), asset.pos(), 2)
            ),
        };
        target_assets.push(target_asset);
    }

    Ok(target_assets)
}

fn get_asset_nodes(ag: &AccessGraph) -> Vec<(NodeIndex, &AssetAttributes)> {
    let nodes = ag.get_nodes();
    nodes
        .filter_map(|(i, n)| match n {
            JettyNode::Asset(a) => Some((i, a)),
            _ => None,
        })
        .collect::<Vec<_>>()
}

/// Return a list of assets that match the asset specified in the configuration.
fn get_matching_assets<'a>(
    target: &TargetAsset,
    asset_list: &[(NodeIndex, &'a AssetAttributes)],
) -> Vec<(NodeIndex, &'a AssetAttributes)> {
    // first look for "exact-end" matches. These are assets whose names end with the exact search term
    // if there 1 or more, return them.
    let mut exact_end_match = asset_list
        .iter()
        .filter(|(_, n)| {
            n.name().name_for_string_matching().ends_with(target.name.as_str())
                // if there is a type, it needs to be a match    
                && if let Some(val) = &target.asset_type {
                    n.asset_type().to_string() == *val
                } else {
                    true
                }
        })
        .map(|(i, n)| (i.to_owned(), *n))
        .collect::<Vec<_>>();

    // if there are no matches, try again, but without case sensitivity
    if exact_end_match.is_empty() {
        exact_end_match = asset_list
        .iter()
        .filter(|(_, n)| {
            n.name().name_for_string_matching().to_lowercase().ends_with(target.name.to_lowercase().as_str())
                // if there is a type, it needs to be a match    
                && if let Some(val) = &target.asset_type {
                    n.asset_type().to_string() == *val
                } else {
                    true
                }
        })
        .map(|(i, n)| (i.to_owned(), *n))
        .collect::<Vec<_>>();
    }

    if exact_end_match.len() == 1 {
        exact_end_match
    }
    // if exact_end_match doesn't find any matches, we look for the term anywhere inside the word
    // This part is case insensitive.
    else {
        asset_list
            .iter()
            .filter(|(_, n)| {
                n.name()
                    .name_for_string_matching()
                    .to_lowercase()
                    .contains(target.name.to_lowercase().as_str())
                    && if let Some(val) = &target.asset_type {
                        n.asset_type().to_string() == *val
                    } else {
                        true
                    }
            })
            .map(|(i, n)| (i.to_owned(), *n))
            .collect::<Vec<_>>()
    }
}

/// Given a list of target assets and a list of all existing assets, return a tuple of AssetMatchErrors and Strings
/// The strings are used to populate a nodes::Tag object which can then be added to the graph.
fn get_asset_list_from_target_list(
    target_list: &Vec<TargetAsset>,
    asset_list: &[(NodeIndex, &AssetAttributes)],
) -> (Vec<AssetMatchError>, HashSet<NodeName>) {
    let mut errors = vec![];
    let mut results = HashSet::new();

    for asset in target_list {
        let matching_assets = get_matching_assets(asset, asset_list);
        // if there are too many matching assets
        if matching_assets.len() > 1 {
            errors.push(AssetMatchError {
                target: asset.to_owned(),
                problem: AssetMatchProblem::TooMany(
                    matching_assets
                        .iter()
                        .map(|(_, a)| -> TargetAsset { (*a).to_owned().into() })
                        .collect(),
                ),
                pos: asset.pos,
            })
        }
        // if there are no matching assets
        else if matching_assets.is_empty() {
            let asset_sans_type = TargetAsset {
                name: asset.name.to_owned(),
                asset_type: None,
                pos: 0,
            };
            // try finding matches with different types
            let untyped_matches = get_matching_assets(&asset_sans_type, asset_list);
            if untyped_matches.len() > 1 {
                errors.push(AssetMatchError {
                    target: asset.to_owned(),
                    problem: AssetMatchProblem::BadType(
                        untyped_matches
                            .iter()
                            .map(|(_, a)| -> TargetAsset { (*a).to_owned().into() })
                            .collect(),
                    ),
                    pos: asset.pos,
                });
            } else {
                errors.push(AssetMatchError {
                    target: asset.to_owned(),
                    problem: AssetMatchProblem::NoMatches,
                    pos: asset.pos,
                });
            }
        }
        // otherwise there was just one match, and we should use it
        else {
            results.insert(matching_assets[0].1.name().to_owned());
        }
    }
    (errors, results)
}

pub(crate) fn tags_to_jetty_node_helpers(
    tags: HashMap<String, TagConfig>,
    ag: &AccessGraph,
    config: &String,
) -> Result<Vec<ProcessedTag>> {
    let mut error_vec = vec![];
    let mut result_vec = vec![];

    let asset_list: Vec<(NodeIndex, &AssetAttributes)> = get_asset_nodes(ag);

    for (tag_name, tag_config) in tags {
        let mut result_tag = ProcessedTag {
            name: NodeName::Tag(tag_name),
            value: tag_config.value,
            description: tag_config.description,
            pass_through_hierarchy: tag_config.pass_through_hierarchy,
            pass_through_lineage: tag_config.pass_through_lineage,
            applied_to: Default::default(),
            removed_from: Default::default(),
            governed_by: Default::default(),
            connector: ConnectorNamespace("Jetty".to_owned()),
        };

        if let Some(target_list) = tag_config.apply_to {
            let (apply_to_errors, apply_to_names) =
                get_asset_list_from_target_list(&target_list, &asset_list);
            error_vec.extend(apply_to_errors);
            result_tag.applied_to = apply_to_names;
        }
        if let Some(target_list) = tag_config.remove_from {
            let (remove_from_errors, remove_from_names) =
                get_asset_list_from_target_list(&target_list, &asset_list);
            error_vec.extend(remove_from_errors);
            result_tag.removed_from = remove_from_names;
        }

        if error_vec.is_empty() {
            result_vec.push(result_tag)
        }
    }
    if !error_vec.is_empty() {
        let error_message = error_vec
            .iter()
            .map(|e| {
                format!(
                    "error at {}\n{}\n",
                    indicated_msg(config.as_bytes(), e.pos, 2),
                    e
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");
        bail!(error_message);
    }
    Ok(result_vec)
}

#[cfg(test)]
mod test {

    use crate::access_graph::cual_to_asset_name_test;

    use crate::cual::Cual;

    use super::*;

    #[test]
    fn parsing_tags_works() -> Result<()> {
        let config = r#"
        pii:
            description: This data contains pii from ppis
            value: I don't know if we want values, but Snowflake has them
            apply_to:
                - snow:jetty_test_db/pizza_schema/"Special Table"
                - tab:project1/project2/workbook1
                - tab:workbook2
                - name: tab:project1/project2/ambiguous
                type:
                    - tableau:workbook
            remove_from:
                - tab:project1/project2/pizza
"#;

        parse_tags(&config.to_owned()).map(|_| ())
    }

    #[test]
    fn ambiguous_asset_name_error_works() -> Result<()> {
        let ag = AccessGraph::new_dummy(
            &[
                &JettyNode::Asset(AssetAttributes::new(
                    Cual::new("asset1://a/asset1"),
                    ConnectorNamespace("cn1".to_owned()),
                )),
                &JettyNode::Asset(AssetAttributes::new(
                    Cual::new("asset2://a/asset1"),
                    ConnectorNamespace("cn2".to_owned()),
                )),
            ],
            &[],
        );

        let config = r#"
        pii:
            description: This data contains pii from ppis
            value: I don't know if we want values, but Snowflake has them
            apply_to:
                - asset
"#
        .to_owned();

        let tag_map = parse_tags(&config)?;
        let t = tags_to_jetty_node_helpers(tag_map, &ag, &config);

        match t {
            Ok(_tags) => bail!("should have returned an error"),
            Err(e) => {
                if e.to_string().contains("unable to disambiguate asset") {
                    Ok(())
                } else {
                    bail!("improper error")
                }
            }
        }
    }

    #[test]
    fn end_match_works() -> Result<()> {
        let ag = AccessGraph::new_dummy(
            &[
                &JettyNode::Asset(AssetAttributes::new(
                    Cual::new("asset1://a/a"),
                    Default::default(),
                )),
                &JettyNode::Asset(AssetAttributes::new(
                    Cual::new("asset2://b/b"),
                    Default::default(),
                )),
            ],
            &[],
        );

        let config = r#"
        pii:
            description: This data contains pii from ppis
            value: I don't know if we want values, but Snowflake has them
            apply_to:
                - a
"#
        .to_owned();

        let tag_map = parse_tags(&config)?;
        tags_to_jetty_node_helpers(tag_map, &ag, &config)?;
        Ok(())
    }

    #[test]
    fn building_tags_works() -> Result<()> {
        let ag = AccessGraph::new_dummy(
            &[
                &JettyNode::Asset(AssetAttributes::new(
                    Cual::new("asset1://a/a1"),
                    Default::default(),
                )),
                &JettyNode::Asset(AssetAttributes::new(
                    Cual::new("asset2://a/a2"),
                    Default::default(),
                )),
            ],
            &[],
        );

        let config = r#"
pii:
    description: This data contains pii from ppis
    value: I don't know if we want values, but Snowflake has them
    apply_to:
        - a1
    remove_from:
        - a2
pii2:
    pass_through_lineage: true
    apply_to:
        - name: a1
          asset_type: ""
"#
        .to_owned();

        let mut goal = vec![
            ProcessedTag {
                name: NodeName::Tag("pii".to_owned()),
                description: Some("This data contains pii from ppis".to_owned()),
                value: Some("I don't know if we want values, but Snowflake has them".to_owned()),
                applied_to: HashSet::from([cual_to_asset_name_test(
                    Cual::new("asset1://a/a1"),
                    Default::default(),
                )]),
                removed_from: HashSet::from([cual_to_asset_name_test(
                    Cual::new("asset2://a/a2"),
                    Default::default(),
                )]),
                connector: ConnectorNamespace("Jetty".to_owned()),
                ..Default::default()
            },
            ProcessedTag {
                name: NodeName::Tag("pii2".to_owned()),
                pass_through_lineage: true,
                applied_to: HashSet::from([cual_to_asset_name_test(
                    Cual::new("asset1://a/a1"),
                    Default::default(),
                )]),
                connector: ConnectorNamespace("Jetty".to_owned()),
                ..Default::default()
            },
        ];

        goal.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());

        let tag_map = parse_tags(&config)?;
        let mut t = tags_to_jetty_node_helpers(tag_map, &ag, &config)?;

        t.sort_by(|a, b| a.name.partial_cmp(&b.name).unwrap());

        assert_eq!(t, goal);

        Ok(())
    }
}
