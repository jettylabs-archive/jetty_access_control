// Convenience import
pub(crate) use jetty_core::cual::{Cual, Cualable};

use crate::manifest::node::NamePartable;

/// Create a CUAL for a set of identifiers.
#[macro_export]
macro_rules! cual {
    ($db:expr) => {
        Cual::new(format!("{}://{}", "snowflake", urlencoding::encode(&$db)))
    };
    ($db:expr, $schema:expr) => {
        Cual::new(format!(
            "{}://{}/{}",
            "snowflake",
            urlencoding::encode(&$db),
            urlencoding::encode(&$schema)
        ))
    };
    ($db:expr, $schema:expr, $table:expr) => {
        Cual::new(format!(
            "{}://{}/{}/{}",
            "snowflake",
            urlencoding::encode(&$db),
            urlencoding::encode(&$schema),
            urlencoding::encode(&$table)
        ))
    };
}

pub(crate) use cual;

impl Cualable for dyn NamePartable {
    fn cual(&self) -> Cual {
        self.dbt_cual()
    }
}

#[cfg(test)]
mod test {
    use jetty_core::connectors::AssetType;

    use crate::manifest::node::{DbtModelNode, DbtSourceNode};

    use super::*;

    #[test]
    fn macro_works() {
        let c = cual!("my_db").uri();
        assert_eq!(c, "snowflake://my_db")
    }

    #[test]
    fn proper_model_node_yields_cual() {
        let result_cual = (&DbtModelNode {
            name: "db.schema.model".to_owned(),
            materialized_as: AssetType::DBTable,
            ..Default::default()
        } as &dyn NamePartable)
            .cual();

        assert_eq!(
            result_cual,
            Cual::new("snowflake://DB/SCHEMA/MODEL".to_owned())
        );
    }

    #[test]
    fn no_quoting_config_yields_no_quotes() {
        let source_node = DbtSourceNode {
            name: r#"db.schema.model"#.to_owned(),
        };

        // No quoting
        let result_cual = (&source_node as &dyn NamePartable).cual();
        assert_eq!(
            result_cual,
            Cual::new("snowflake://DB/SCHEMA/MODEL".to_owned())
        );
    }

    #[test]
    fn db_quoting_config_results_in_quotes() {
        let source_node = DbtSourceNode {
            name: r#"\"db\".schema.model"#.to_owned(),
        };
        // Just db
        let result_cual = (&source_node as &dyn NamePartable).cual();
        dbg!(&result_cual);
        assert_eq!(
            result_cual,
            Cual::new("snowflake://db/SCHEMA/MODEL".to_owned())
        );
    }

    #[test]
    fn schema_quoting_config_results_in_quotes() {
        let source_node = DbtSourceNode {
            name: r#"db.\"schema\".model"#.to_owned(),
        };
        // Just schema
        let result_cual = (&source_node as &dyn NamePartable).cual();
        assert_eq!(
            result_cual,
            Cual::new("snowflake://DB/schema/MODEL".to_owned())
        );
    }

    #[test]
    fn identifier_quoting_config_results_in_quotes() {
        let source_node = DbtSourceNode {
            name: r#"db.schema.\"model\""#.to_owned(),
        };
        // Just identifier
        let result_cual = (&source_node as &dyn NamePartable).cual();
        assert_eq!(
            result_cual,
            Cual::new("snowflake://DB/SCHEMA/model".to_owned())
        );
    }

    #[test]
    fn db_schema_quoting_config_results_in_quotes() {
        let source_node = DbtSourceNode {
            name: r#"\"db\".\"schema\".model"#.to_owned(),
        };
        // db and schema
        let result_cual = (&source_node as &dyn NamePartable).cual();
        assert_eq!(
            result_cual,
            Cual::new("snowflake://db/schema/MODEL".to_owned())
        );
    }

    #[test]
    fn db_schema_identifier_quoting_config_results_in_quotes() {
        let source_node = DbtSourceNode {
            name: r#"\"db\".\"schema\".\"model\""#.to_owned(),
        };
        // db and schema and identifier
        let result_cual = (&source_node as &dyn NamePartable).cual();
        assert_eq!(
            result_cual,
            Cual::new("snowflake://db/schema/model".to_owned())
        );
    }

    #[test]
    fn db_identifier_quoting_config_results_in_quotes() {
        let source_node = DbtSourceNode {
            name: r#"\"db\".schema.\"model\""#.to_owned(),
        };
        // db and schema and identifier
        let result_cual = (&source_node as &dyn NamePartable).cual();
        assert_eq!(
            result_cual,
            Cual::new("snowflake://db/SCHEMA/model".to_owned())
        );
    }

    #[test]
    fn schema_identifier_quoting_config_results_in_quotes() {
        let source_node = DbtSourceNode {
            name: r#"db.\"schema\".\"model\""#.to_owned(),
        };
        // db and schema and identifier
        let result_cual = (&source_node as &dyn NamePartable).cual();
        assert_eq!(
            result_cual,
            Cual::new("snowflake://DB/schema/model".to_owned())
        );
    }

    /// Periods between quotes aren't currently supported.
    #[test]
    #[should_panic]
    fn periods_in_quotes_panics() {
        let source_node = DbtSourceNode {
            name: r#"db.\"schema.schema2\".\"model\""#.to_owned(),
        };
        // db and schema and identifier
        let result_cual = (&source_node as &dyn NamePartable).cual();
        assert_eq!(
            result_cual,
            Cual::new("snowflake://DB/schema/model".to_owned())
        );
    }
}
