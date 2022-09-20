use std::collections::{HashMap, HashSet};

use anyhow::Result;

use super::{snowflake_common, NamedConnection};

/// Represents the different types of relations that we can parse
#[derive(Hash, PartialEq, Eq, Debug)]
enum Relation {
    SnowflakeTable(snowflake_common::SnowflakeTableInfo),
    SnowflakeQuery(snowflake_common::SnowflakeQueryInfo),
}

/// This Macro implements to_cuals for Relation by matching on
/// the inner enum types
macro_rules! impl_to_cuals {
    ($($t:tt),+) => {
        impl Relation {
            fn to_cuals(&self, connections: &HashMap<String, NamedConnection>) -> Result<Vec<String>> {
                match self {
                    $(Relation::$t(n) => n.to_cuals(connections),)*
                }
            }
        }
    }
}

impl_to_cuals!(SnowflakeTable, SnowflakeQuery);

/// Gets cuals from an xml file by parsing the file, pulling out the relevant relations,
/// and building an identifier from it.
#[allow(unused)]
pub(super) fn get_cuals_from_datasource(data: &str) -> Result<HashSet<String>> {
    let doc = roxmltree::Document::parse(data).unwrap();

    // filter the doc down to the connection info
    let connection_info = doc
        .descendants()
        .find(|n| n.has_tag_name("connection"))
        .unwrap();

    // pull out the named connections - we'll use these later to get info needed down the road.
    let named_connections = get_named_connections(connection_info);

    // pull out the relations
    let relations = get_relations(connection_info);

    let mut cuals = HashSet::new();
    for r in relations {
        let c = r.to_cuals(&named_connections).unwrap_or_else(|e| {
            println!("unable to create qual from {:#?}", r);
            vec![]
        });

        cuals.extend(c);
    }

    Ok(cuals)
}

/// Given a node, look at the children and pull out named connection information.
/// Currently only looks for Snowflake connections.
fn get_named_connections(node: roxmltree::Node) -> HashMap<String, NamedConnection> {
    let mut named_connections = HashMap::new();
    for n in node.descendants() {
        if n.is_element() && n.has_tag_name("named-connection") {
            if let Some(c) = snowflake_common::try_snowflake_named_conn(&n) {
                named_connections
                    .insert(c.name.to_owned(), NamedConnection::Snowflake(c.to_owned()));
            }
        }
    }
    named_connections
}

/// Given an XML node, find the embedded relations. It takes multiple passes over the descendants
/// of `node`, but this is generally fast enough not to cause any major issues.
fn get_relations(node: roxmltree::Node) -> HashSet<Relation> {
    let mut relations = HashSet::new();
    // start with queries
    node.descendants()
        .filter(|n| {
            n.has_tag_name("relation")
                && n.attribute("name").unwrap_or_else(|| "false") == "Custom SQL Query".to_owned()
        })
        .filter_map(|n| snowflake_common::try_snowflake_query(&n))
        .map(|q| relations.insert(Relation::SnowflakeQuery(q)));

    // now get tables
    let tables: HashSet<_> = node
        .descendants()
        .filter(|n| {
            n.has_tag_name("relation")
                && n.attribute("type").unwrap_or_else(|| "false") == "table".to_owned()
        })
        .collect();

    for table in tables {
        if let Some(t) = snowflake_common::try_snowflake_table(&table) {
            relations.insert(Relation::SnowflakeTable(t));
        };
    }

    relations
}
