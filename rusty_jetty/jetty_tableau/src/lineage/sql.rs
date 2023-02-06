//! SQL-related functionality for tableau lineage gathering

use anyhow::{anyhow, bail, Result};
use sqlparser::{
    ast::{Ident, SelectItem, SetExpr, Statement},
    dialect::GenericDialect,
    parser::Parser,
};

/// Given an identifier name from Tableau, parse it and return the a vector of name parts
pub(crate) fn parse_identifier(ident: &str) -> Result<Vec<String>> {
    let formatted_name = format_tableau_table_name(ident);
    let identifier_expr = get_identifier(formatted_name)?;
    Ok(capitalize_and_split_identifier(identifier_expr))
}

/// Turn a table name to a string
fn format_tableau_table_name(name: &str) -> String {
    if name.split("].[").count() > 1 {
        name.split("].[")
            .map(|identifier| {
                let identifier = identifier.strip_prefix('[').unwrap_or(identifier);
                let identifier = identifier.strip_suffix(']').unwrap_or(identifier);
                let identifier = identifier.replace('"', r#""""#);
                format!("\"{identifier}\"")
            })
            .collect::<Vec<_>>()
            .join(".")
    } else {
        name.to_string()
    }
}

/// A hacky little function to get an Ident from a db-name
fn get_identifier(name: String) -> Result<sqlparser::ast::Expr> {
    let dialect = GenericDialect {}; // or AnsiDialect

    let sql = format!("SELECT {name} from table_1;");

    let ast = &match Parser::parse_sql(&dialect, &sql) {
        Ok(s) => s,
        Err(_) => bail!("unable to parse identifier name for {name}"),
    }[0];
    if let Statement::Query(q) = ast {
        let q = q.as_ref();
        if let SetExpr::Select(s) = &*q.body {
            let item = s
                .projection
                .get(0)
                .ok_or_else(|| anyhow!("didn't find identifer"))?;
            let compound_id =
                if let SelectItem::UnnamedExpr(sqlparser::ast::Expr::Identifier(i)) = item {
                    
                    sqlparser::ast::Expr::CompoundIdentifier(vec![i.to_owned()])
                } else if let SelectItem::UnnamedExpr(i) = item {
                    i.to_owned()
                } else {
                    bail!("unreadable table name");
                };
            return Ok(compound_id);
        }
    }
    bail!("didn't find identifer");
}

fn capitalize_and_split_identifier(identifier: sqlparser::ast::Expr) -> Vec<String> {
    let ident_vec = match &identifier {
        sqlparser::ast::Expr::CompoundIdentifier(i) => i
            .to_owned()
            .iter_mut()
            .map(|i: &mut Ident| {
                if i.quote_style.is_none() {
                    i.value = i.value.to_uppercase();
                }
                i.quote_style = Some('"');
                let mut quoted_name = i.to_string();
                quoted_name.pop(); // remove last
                if !quoted_name.is_empty() {
                    quoted_name.remove(0); // remove first
                };
                quoted_name
            })
            .collect::<Vec<_>>(),
        _ => panic!("expected compound identifier"),
    };
    ident_vec
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_and_capitalize_identifier() -> Result<()> {
        assert_eq!(
            capitalize_and_split_identifier(get_identifier(
                r#"bob."""Special Name""""#.to_owned()
            )?),
            vec!["BOB".to_owned(), r#"""Special Name"""#.to_owned()]
        );
        Ok(())
    }

    #[test]
    fn test_format_tableau_table_name() -> Result<()> {
        assert_eq!(
            format_tableau_table_name(r#"bob."""Special Name""""#),
            r#"bob."""Special Name""""#.to_owned()
        );
        assert_eq!(
            format_tableau_table_name(r#"[RAW].["Special Name"]"#),
            r#""RAW"."""Special Name""""#.to_owned()
        );
        assert_eq!(
            format_tableau_table_name("[GOLD].[IRIS_JOINED_TABLE]"),
            r#""GOLD"."IRIS_JOINED_TABLE""#.to_owned()
        );

        Ok(())
    }
}
