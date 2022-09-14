mod node;

use std::collections::HashSet;

use sqlparser::ast::{self as parser_ast};
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

use node::Node;

/// Parse a SQL query and extract db table names from the query.
pub fn get_tables(query: &str) -> HashSet<String> {
    let dialect = GenericDialect {}; // or AnsiDialect

    let root = node::Node::Statement(Parser::parse_sql(&dialect, query).unwrap()[0].to_owned());

    // Get query
    let descendants = root.get_descendants();
    let query_node = descendants.iter().find(|n| matches!(n, Node::Query(_)));

    if let Some(node::Node::Query(parser_ast::Query { body, .. })) = query_node {
        let body = Node::SetExpr(*body.to_owned());
        let descendants = body.get_descendants();
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
        let table_names: HashSet<String> = object_names
            .iter()
            .map(|o| {
                o.0.iter()
                    .map(|i| i.value.to_owned())
                    .collect::<Vec<String>>()
                    .join(".")
            })
            .collect();
        table_names
    } else {
        HashSet::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    use std::collections::HashSet;

    #[test]
    fn get_table_from_simple_query() -> Result<()> {
        let cases = [("SELECT * FROM a.b".to_owned(), ["a.b".to_owned()])];

        for case in cases {
            let results = get_tables(&case.0);
            assert_eq!(results, HashSet::from(case.1));
        }
        Ok(())
    }
}
