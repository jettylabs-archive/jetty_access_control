mod ast;
mod node;

use std::collections::HashSet;

use anyhow::Result;
use sqlparser::ast as parser_ast;
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

trait AstWalker {
    fn children(&self) -> Vec<parser_ast::Statement>;

    // fn descendent(&self) -> Vec<parser_ast::Statement> {
    //     results = Vec::new();
    //     if let Some(children) = self.children() {
    //         for child in children {}
    //     } else {
    //         todo!()
    //     }
    //     todo!()
    // }

    fn get_descendent(
        &self,
        descendent: &mut Vec<parser_ast::Statement>,
    ) -> &mut Vec<parser_ast::Statement> {
        for child in self.children() {
            descendent.extend(
                self.get_descendent(descendent)
                    .iter()
                    .collect::<Vec<parser_ast::Statement>>(),
            )
        }
        descendent
    }
}

fn get_tables(query: &String) -> HashSet<String> {
    let dialect = GenericDialect {}; // or AnsiDialect

    let ast = Parser::parse_sql(&dialect, &query).unwrap();

    todo!()
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    use std::collections::HashSet;

    #[test]
    fn get_table_from_simple_query() -> Result<()> {
        let query = "SELECT * FROM a".to_owned();
        let desired_result = HashSet::from(["a".to_owned()]);
        let results = get_tables(&query);

        assert_eq!(results, desired_result);
        Ok(())
    }
}
