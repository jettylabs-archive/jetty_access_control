use regex::Regex;

#[derive(Debug, Clone)]
pub(crate) struct SnowflakeConnectionInfo {
    pub name: String,
    pub db: String,
    pub server: String,
    pub schema: String,
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub(crate) struct SnowflakeTableInfo {
    pub table: String,
    pub connection: String,
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub(crate) struct SnowflakeQueryInfo {
    pub query: String,
    pub connection: String,
}

// NamedConnection comes in
pub(super) fn try_snowflake_named_conn(node: &roxmltree::Node) -> Option<SnowflakeConnectionInfo> {
    if let Some(name) = node.attribute("name") {
        if !name.starts_with("snowflake.") {
            return None;
        }
    } else {
        return None;
    }
    let connection_node = node.children().find(|n| n.has_tag_name("connection"))?;

    Some(SnowflakeConnectionInfo {
        name: node.attribute("name")?.to_owned(),
        db: connection_node.attribute("dbname")?.to_owned(),
        server: connection_node.attribute("server")?.to_owned(),
        schema: connection_node.attribute("schema")?.to_owned(),
    })
}

pub(super) fn try_snowflake_query(node: &roxmltree::Node) -> Option<SnowflakeQueryInfo> {
    let connection = node.attribute("connection")?;
    if !connection.starts_with("snowflake.") {
        return None;
    }

    let re = Regex::new(r"(<\[Parameters\].*.>)").unwrap();

    Some(SnowflakeQueryInfo {
        query: re
            .replace_all(&node.text().unwrap(), "ignore___tableau_parameter")
            .to_string(),
        connection: connection.to_string(),
    })
}

pub(super) fn try_snowflake_table(node: &roxmltree::Node) -> Option<SnowflakeTableInfo> {
    let connection = node.attribute("connection")?;
    if !connection.starts_with("snowflake.") {
        return None;
    }

    Some(SnowflakeTableInfo {
        table: node.attribute("table")?.replace("[", "").replace("]", ""),
        connection: connection.to_string(),
    })
}
