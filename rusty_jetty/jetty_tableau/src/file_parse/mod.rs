pub(crate) mod flow;
pub(crate) mod origin;
mod snowflake_common;
pub(crate) mod xml_docs;

/// Named connection information from the tableau files
#[derive(Debug)]
enum NamedConnection {
    Snowflake(snowflake_common::SnowflakeConnectionInfo),
}

enum RelationType {
    SqlQuery,
    Table,
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use anyhow::Result;

    /// A very basic test to make sure that things don't panic or fail
    #[test]
    fn parse_tables_from_tds_works() -> Result<()> {
        let data = fs::read_to_string("test_data/test1.xml").expect("unable to read file");
        let x = xml_docs::parse(&data)?;
        dbg!(x);
        Ok(())
    }
}
