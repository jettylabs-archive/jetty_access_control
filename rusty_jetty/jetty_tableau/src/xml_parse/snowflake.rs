use std::collections::HashMap;

use anyhow::{anyhow, bail, Result};
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

impl SnowflakeTableInfo {
    pub(super) fn to_cuals(
        &self,
        connections: &HashMap<String, super::NamedConnection>,
    ) -> Result<Vec<String>> {
        let super::NamedConnection::Snowflake(conn) = connections
            .get(&self.connection)
            .ok_or(anyhow!["unable to find connection"])?;

        let prefix = format!["snowflake://{}", &conn.server.to_lowercase()];
        let name_parts: Vec<String> = self
            .table
            .trim_matches(|c| c == '[' || c == ']')
            .split("].[")
            .map(|s| s.to_owned())
            .collect();

        let cual = if name_parts.len() == 3 {
            format!(
                "{}/{}/{}/{}",
                prefix, name_parts[0], name_parts[1], name_parts[2]
            )
        } else if name_parts.len() == 2 {
            format!("{}/{}/{}/{}", prefix, conn.db, name_parts[0], name_parts[1])
        } else if name_parts.len() == 1 {
            format!("{}/{}/{}/{}", prefix, conn.db, conn.schema, name_parts[0])
        } else {
            bail!("unable to build cual")
        };

        Ok(vec![cual])
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn table_to_cuals_correctly() -> Result<()> {
        let connections = HashMap::from([(
            "connection_name".to_owned(),
            crate::xml_parse::NamedConnection::Snowflake(super::SnowflakeConnectionInfo {
                name: "connection_name".to_owned(),
                db: "MY_DB".to_owned(),
                server: "HereSaTest.snowflakecomputing.com".to_owned(),
                schema: "MY_SCHEMA".to_owned(),
            }),
        )]);

        let table_info = SnowflakeTableInfo {
            table: "[MY_SCHEMA].[MY_TABLE]".to_owned(),
            connection: "connection_name".to_owned(),
        };

        let cuals = table_info.to_cuals(&connections)?;

        assert_eq!(
            cuals,
            vec!["snowflake://heresatest.snowflakecomputing.com/MY_DB/MY_SCHEMA/MY_TABLE"]
        );

        Ok(())
    }
}
