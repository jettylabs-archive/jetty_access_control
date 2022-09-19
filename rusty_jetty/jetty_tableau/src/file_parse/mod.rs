mod flow;
mod snowflake_common;
mod xml_docs;

use std::collections::HashMap;

use anyhow::Result;

/// Named connection information from the tableau files
enum NamedConnection {
    Snowflake(snowflake_common::SnowflakeConnectionInfo),
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
        let x = xml_docs::get_cuals_from_datasource(&data)?;
        dbg!(x);
        Ok(())
    }
}
