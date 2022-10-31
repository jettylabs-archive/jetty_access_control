use anyhow::{anyhow, bail, Context, Result};
use jetty_core::{
    cual::Cual,
    logging::{error},
};

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
    pub db: String,
    pub server: String,
    pub schema: String,
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub(crate) struct SnowflakeQueryInfo {
    pub query: String,
    pub db: String,
    pub server: String,
    pub schema: String,
}

impl SnowflakeTableInfo {
    pub(super) fn to_cuals(&self) -> Result<Vec<Cual>> {
        let name_parts = self.get_table_name_parts();

        Ok(
            cual_from_name_parts(&name_parts, &self.server, &self.db, &self.schema).map_or_else(
                |_| {
                    error!("Unable to print create qual from {:#?}", name_parts);
                    vec![]
                },
                |cual| vec![cual],
            ),
        )
    }

    fn get_table_name_parts(&self) -> Vec<String> {
        self.table
            .trim_matches(|c| c == '[' || c == ']')
            .split("].[")
            .map(|s| s.to_owned())
            .collect()
    }
}

impl SnowflakeQueryInfo {
    pub(super) fn to_cuals(&self) -> Result<Vec<Cual>> {
        let relations = jetty_sql::get_tables(&self.query, jetty_sql::DbType::Snowflake)
            .context("parsing query")?;

        let mut cuals = Vec::new();
        for name_parts in relations {
            cual_from_name_parts(&name_parts, &self.server, &self.db, &self.schema).map_or_else(
                |_| {
                    error!("Unable to print create qual from {:#?}", name_parts);
                },
                |cual| cuals.push(cual),
            )
        }
        Ok(cuals)
    }
}

fn cual_from_name_parts(
    name_parts: &[String],
    server: &str,
    db: &str,
    schema: &str,
) -> Result<Cual> {
    let name_parts: Vec<std::borrow::Cow<str>> =
        name_parts.iter().map(|p| urlencoding::encode(p)).collect();

    let prefix = format![
        "snowflake://{}",
        urlencoding::encode(&server.to_lowercase())
    ];

    let cual = match name_parts.len() {
        3 => format!(
            "{}/{}/{}/{}",
            prefix, name_parts[0], name_parts[1], name_parts[2]
        ),
        2 => format!("{prefix}/{db}/{}/{}", name_parts[0], name_parts[1]),
        1 => format!("{prefix}/{db}/{schema}/{}", name_parts[0]),
        _ => bail!("unable to build cual"),
    };
    Ok(Cual::new(&cual))
}

/// Build a named SnowflakeConnectionInfo instance for a snowflake source
pub(super) fn build_snowflake_connection_info(
    node: &roxmltree::Node,
) -> Result<SnowflakeConnectionInfo> {
    let connection_node = node
        .children()
        .find(|n| n.has_tag_name("connection"))
        .ok_or_else(|| anyhow!("unable to find connection information for snowflake source"))?;

    Ok(SnowflakeConnectionInfo {
        name: node
            .attribute("name")
            .ok_or_else(|| anyhow!("unable to find connection information for snowflake source"))?
            .to_owned(),
        db: connection_node
            .attribute("dbname")
            .ok_or_else(|| anyhow!("unable to find connection information for snowflake source"))?
            .to_owned(),
        server: connection_node
            .attribute("server")
            .ok_or_else(|| anyhow!("unable to find connection information for snowflake source"))?
            .to_owned(),
        schema: connection_node
            .attribute("schema")
            .ok_or_else(|| anyhow!("unable to find connection information for snowflake source"))?
            .to_owned(),
    })
}

#[cfg(test)]
mod tests {

    use super::*;
    use anyhow::Result;

    #[test]
    fn table_to_cuals_correctly() -> Result<()> {
        let table_info = SnowflakeTableInfo {
            table: "[MY_SCHEMA].[MY_TABLE]".to_owned(),
            db: "MY_DB".to_owned(),
            server: "HereSaTest.snowflakecomputing.com".to_owned(),
            schema: "MY_SCHEMA".to_owned(),
        };

        let cuals = table_info.to_cuals()?;

        assert_eq!(
            cuals,
            vec![Cual::new(
                "snowflake://heresatest.snowflakecomputing.com/MY_DB/MY_SCHEMA/MY_TABLE"
            )]
        );

        Ok(())
    }
}
