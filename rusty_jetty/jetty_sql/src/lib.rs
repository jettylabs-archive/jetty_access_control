//! SQL Parsing utilities for Jetty!
//!

#![deny(missing_docs)]

mod node;

use std::collections::{HashMap, HashSet};
use std::ops::Deref;

use anyhow::{bail, Result};
use sqlparser::ast;
use sqlparser::dialect::{self, Dialect};
use sqlparser::parser::Parser;

use node::Node;

type TableName = Vec<String>;

/// The type of database the query is for.
pub enum DbType {
    /// Snowflake CDW
    Snowflake,
    /// Generic SQL
    Generic,
}

/// Parse a SQL query and extract db table names from the query. Returns a vector of
/// fully-qualified path components, eg: ["db_name", "schema_name", "table_name"]
pub fn get_tables(query: &str, db: DbType) -> Result<HashSet<TableName>> {
    let dialect: Box<dyn Dialect> = match db {
        DbType::Snowflake => Box::new(dialect::SnowflakeDialect {}),
        DbType::Generic => Box::new(dialect::GenericDialect {}),
    };

    let root = node::Node::Statement(Parser::parse_sql(dialect.deref(), query)?[0].to_owned());
    if !matches!(&root, node::Node::Statement(ast::Statement::Query(_))) {
        bail!("not a query - skipping")
    }

    // Get query
    let descendants = root.get_descendants();
    let query_node = descendants.iter().find(|n| matches!(n, Node::Query(_)));

    if let Some(node::Node::Query(ast::Query { body, with, .. })) = query_node {
        // Tables are identified by a vec of name parts e.g., ["schema_name", "table_name"]
        // The cte context is a lookup map that is built up as CTEs are processed. When a table
        // name is found in a query, it's checked against this context, and if the table matches
        // a CTE, the source tables from the cte (rather than the CTE's name) are inserted into
        // the final source list
        let mut cte_context: HashMap<TableName, HashSet<TableName>> = HashMap::new();
        if let Some(with_node) = with {
            for cte in &with_node.cte_tables {
                let table_name: TableName = vec![capitalize_identifiers(&cte.alias.name, &db)];
                let sources = get_tables_from_node(&Node::Cte(cte.to_owned()), &cte_context, &db);
                cte_context.insert(table_name, sources);
            }
        }

        let body = Node::SetExpr(*body.to_owned());
        Ok(get_tables_from_node(&body, &cte_context, &db))
    } else {
        Ok(HashSet::new())
    }
}

/// Given Node, get the referenced tables. Looks up against a cte_context variable to
/// make sure to correctly identify source tables
fn get_tables_from_node(
    node: &Node,
    cte_context: &HashMap<TableName, HashSet<TableName>>,
    db: &DbType,
) -> HashSet<TableName> {
    let descendants = node.get_descendants();
    let object_names: Vec<ast::ObjectName> = descendants
        .iter()
        // filter to the TableFactor::Tables and then get the name
        .filter_map(|n| {
            if let Node::TableFactor(ast::TableFactor::Table { name, .. }) = n {
                Some(name.to_owned())
            } else {
                None
            }
        })
        .collect();
    let table_names: HashSet<TableName> = object_names
        .iter()
        .map(|o| {
            o.0.iter()
                .map(|i| capitalize_identifiers(i, db))
                .collect::<TableName>()
        })
        .flat_map(|n| match cte_context.get(&n) {
            Some(v) => v.to_owned(),
            None => HashSet::from([n.to_owned()]),
        })
        .collect();

    table_names
}

/// Capitalize identifiers appropriately, given a DbType
fn capitalize_identifiers(i: &ast::Ident, db: &DbType) -> String {
    match db {
        DbType::Snowflake => {
            if i.quote_style.is_some() {
                i.value.to_owned()
            } else {
                i.value.to_uppercase()
            }
        }
        DbType::Generic => {
            if i.quote_style.is_some() {
                i.value.to_owned()
            } else {
                i.value.to_lowercase()
            }
        }
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
                "SELECT * FROM a.b where pizza='abc'".to_owned(),
                vec![vec!["A".to_owned(), "B".to_owned()]],
            ),
            (
                "SELECT * FROM A.B".to_owned(),
                vec![vec!["A".to_owned(), "B".to_owned()]],
            ),
            (
                r#"SELECT * FROM "test".B"#.to_owned(),
                vec![vec!["test".to_owned(), "B".to_owned()]],
            ),
            (
                r#"with a as (select z from schema_a.table_1) Select * FROM a"#.to_owned(),
                vec![vec!["SCHEMA_A".to_owned(), "TABLE_1".to_owned()]],
            ),
            (
            r#"with a as (select z from schema_a.table_1) Select * FROM b"#.to_owned(),
            vec![vec!["B".to_owned()]],
            ),
            (
                r#"with a as (select z from schema_a.table_1), b as (select z from a) Select * FROM b"#.to_owned(),
                vec![vec!["SCHEMA_A".to_owned(), "TABLE_1".to_owned()]],
            ),
            (
                r#"with a as (select z from schema_a.table_1), b as (select z from a) Select * FROM b left join c.d using(z)"#.to_owned(),
                vec![vec!["SCHEMA_A".to_owned(), "TABLE_1".to_owned()], vec!["C".to_owned(), "D".to_owned()]],
            ),
        ];

        for case in cases {
            let results = get_tables(&case.0, DbType::Snowflake)?;
            assert_eq!(results, HashSet::from_iter(case.1.into_iter()));
        }
        Ok(())
    }
}
