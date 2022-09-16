mod snowflake;

use std::collections::{HashMap, HashSet};

use anyhow::Result;

enum NamedConnection {
    Snowflake(snowflake::SnowflakeConnectionInfo),
}

#[derive(Hash, PartialEq, Eq, Debug)]
enum Relation {
    SnowflakeTable(snowflake::SnowflakeTableInfo),
    SnowflakeQuery(snowflake::SnowflakeQueryInfo),
}

impl Relation {
    fn to_cual(&self, connections: &HashMap<String, NamedConnection>) -> String {
        todo!()
    }
}
// Impl to_cual on Relation
// Also, convert a bunch of this to methods

pub(crate) fn parse_tds_sources() {
    todo!()
}

pub(crate) fn parse_twb_sources() {
    todo!()
}

pub(crate) fn parse_flow_sources() {
    todo!()
}

fn parse_standard_datasource(data: &str) -> Result<()> {
    let doc = roxmltree::Document::parse(data).unwrap();

    // filter the doc down to the connection info
    let connection_info = doc
        .descendants()
        .find(|n| n.has_tag_name("connection"))
        .unwrap();

    // pull out the named connections - we'll use these later to get info needed down the road.
    let named_connections = get_named_conections(connection_info);

    // pull out the relations
    let relations = get_relations(connection_info);

    let cuals: Vec<String> = relations
        .into_iter()
        .map(|r| r.to_cual(&named_connections))
        .collect();

    todo!()
}

/// Given a node, look at the children and pull out named connection information.
/// Currently only looks for Snowflake connections.
fn get_named_conections(node: roxmltree::Node) -> HashMap<String, NamedConnection> {
    let mut named_connections = HashMap::new();
    for n in node.descendants() {
        if n.is_element() && n.has_tag_name("named-connection") {
            if let Some(c) = snowflake::try_snowflake_named_conn(&n) {
                named_connections
                    .insert(c.name.to_owned(), NamedConnection::Snowflake(c.to_owned()));
            }
        }
    }
    named_connections
}

fn get_relations(node: roxmltree::Node) -> HashSet<Relation> {
    let mut relations = HashSet::new();
    // start with queries
    let queries: HashSet<_> = node
        .descendants()
        .filter(|n| {
            n.has_tag_name("relation")
                && n.attribute("name").unwrap_or_else(|| "false") == "Custom SQL Query".to_owned()
        })
        .collect();

    for query in queries {
        if let Some(q) = snowflake::try_snowflake_query(&query) {
            relations.insert(Relation::SnowflakeQuery(q));
        };
    }

    // now get tables
    let tables: HashSet<_> = node
        .descendants()
        .filter(|n| {
            n.has_tag_name("relation")
                && n.attribute("type").unwrap_or_else(|| "false") == "table".to_owned()
        })
        .collect();

    for table in tables {
        if let Some(t) = snowflake::try_snowflake_table(&table) {
            relations.insert(Relation::SnowflakeTable(t));
        };
    }

    relations
}
