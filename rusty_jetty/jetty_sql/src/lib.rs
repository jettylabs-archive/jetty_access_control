mod node;

use std::collections::HashSet;
use std::ops::Deref;

use anyhow::Result;
use sqlparser::ast::{self as parser_ast};
use sqlparser::dialect::{self, Dialect};
use sqlparser::parser::Parser;

use node::Node;

pub enum DbType {
    Snowflake,
    Generic,
}

/// Parse a SQL query and extract db table names from the query. Returns a vector of
/// fully-qualified path components, eg: ["db_name", "schema_name", "table_name"]
pub fn get_tables(query: &str, db: DbType) -> Result<HashSet<Vec<String>>> {
    let dialect: Box<dyn Dialect> = match db {
        DbType::Snowflake => Box::new(dialect::SnowflakeDialect {}),
        DbType::Generic => Box::new(dialect::GenericDialect {}),
    };

    let capitalize_identifiers = |i: &parser_ast::Ident| match db {
        DbType::Snowflake => {
            if i.quote_style.is_some() {
                i.value.to_owned()
            } else {
                i.value.to_uppercase().to_owned()
            }
        }
        DbType::Generic => {
            if i.quote_style.is_some() {
                i.value.to_owned()
            } else {
                i.value.to_lowercase().to_owned()
            }
        }
    };

    let root =
        node::Node::Statement(Parser::parse_sql(dialect.deref(), query).unwrap()[0].to_owned());

    // Get query
    let descendants = root.get_descendants()?;
    let query_node = descendants.iter().find(|n| matches!(n, Node::Query(_)));

    if let Some(node::Node::Query(parser_ast::Query { body, .. })) = query_node {
        let body = Node::SetExpr(*body.to_owned());
        let descendants = body.get_descendants()?;
        let object_names: Vec<parser_ast::ObjectName> = descendants
            .iter()
            .filter_map(|n| {
                if let Node::TableFactor(parser_ast::TableFactor::Table { name, .. }) = n {
                    Some(name.to_owned())
                } else {
                    None
                }
            })
            .collect();
        let table_names: HashSet<Vec<String>> = object_names
            .iter()
            .map(|o| {
                o.0.iter()
                    .map(capitalize_identifiers)
                    .collect::<Vec<String>>()
            })
            .collect();
        Ok(table_names)
    } else {
        Ok(HashSet::new())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    use std::collections::HashSet;

    #[test]
    fn get_table_from_simple_snowflake_queries() -> Result<()> {
        let cases = [
            (
                "SELECT * FROM a.b".to_owned(),
                [vec!["A".to_owned(), "B".to_owned()]],
            ),
            (
                "SELECT * FROM A.B".to_owned(),
                [vec!["A".to_owned(), "B".to_owned()]],
            ),
            (
                r#"SELECT * FROM "test".B"#.to_owned(),
                [vec!["test".to_owned(), "B".to_owned()]],
            ),
        ];

        for case in cases {
            let results = get_tables(&case.0, DbType::Snowflake)?;
            assert_eq!(results, HashSet::from(case.1));
        }
        Ok(())
    }
}
