use jetty_core::{
    connectors::AssetType,
    cual::{Cual, Cualable},
};

use crate::manifest::node::{DbtModelNode, DbtNode, DbtSourceNode, QuotingConfig};

/// Create a qual for a set of identifiers.
///
/// The pattern here is identifier => should_quote.
macro_rules! cual {
    ($db:expr => $quote_db:expr) => {
        Cual::new(format!(
            "{}://{}",
            "snowflake",
            urlencoding::encode(&quote_str!($db, $quote_db)),
        ))
    };
    ($db:expr => $quote_db:expr, $schema:expr => $quote_schema:expr) => {
        Cual::new(format!(
            "{}://{}/{}",
            "snowflake",
            urlencoding::encode(&quote_str!($db, $quote_db)),
            urlencoding::encode(&quote_str!($schema, $quote_schema)),
        ))
    };
    ($db:expr => $quote_db:expr, $schema:expr => $quote_schema:expr, $table:expr => $quote_table:expr) => {
        Cual::new(format!(
            "{}://{}/{}/{}",
            "snowflake",
            urlencoding::encode(&quote_str(&$db, $quote_db)),
            urlencoding::encode(&quote_str(&$schema, $quote_schema)),
            urlencoding::encode(&quote_str(&$table, $quote_table))
        ))
    };
}

/// Add quotes around the string if `quote`
fn quote_str(s: &str, quote: bool) -> String {
    if quote {
        format!("\"{}\"", s.to_owned())
    } else {
        s.to_owned()
    }
}

impl Cualable for DbtNode {
    fn cual(&self) -> Cual {
        match self {
            DbtNode::ModelNode(DbtModelNode {
                name,
                database,
                schema,
                quoting:
                    QuotingConfig {
                        database: quote_db,
                        schema: quote_schema,
                        identifier: quote_ident,
                    },
                ..
            }) => {
                cual!(database.to_owned() => *quote_db, schema.to_owned() => *quote_schema, name.to_owned() => *quote_ident)
            }
            DbtNode::SourceNode(DbtSourceNode {
                name,
                database,
                schema,
                quoting:
                    QuotingConfig {
                        database: quote_db,
                        schema: quote_schema,
                        identifier: quote_ident,
                    },
                ..
            }) => {
                cual!(database.to_owned() => *quote_db, schema.to_owned() => *quote_schema, name.to_owned() => *quote_ident)
            }
        }
    }
}

impl Cualable for DbtModelNode {
    fn cual(&self) -> Cual {
        // If the model is materialized as a warehouse object, it gets a
        // warehouse-specific CUAL.
        // Otherwise, it gets a dbt CUAL.
        match self.materialized_as {
            AssetType::DBTable | AssetType::DBView => {
                let QuotingConfig {
                    database,
                    schema,
                    identifier,
                } = self.quoting;
                cual!(
                    self.database.to_owned() => database,
                    self.schema.to_owned() => schema,
                    self.name.to_owned() => identifier
                )
            }
            // Every model that gets passed in here should be materialized
            // as a table or view.
            _ => panic!(
                "Failed to get CUAL for dbt node. Wrong materialization: {:?}",
                self.materialized_as
            ),
        }
    }
}

impl Cualable for DbtSourceNode {
    fn cual(&self) -> Cual {
        // Sources come from the db. Create the CUAL to correspond to
        // the origin datastore.
        let QuotingConfig {
            database,
            schema,
            identifier,
        } = self.quoting;

        cual!(&self.database => database, &self.schema => schema, &self.name => identifier)
    }
}

#[cfg(test)]
mod test {
    use crate::manifest::node::{DbtModelNode, QuotingConfig};

    use super::*;

    #[test]
    fn proper_model_node_yields_cual() {
        let result_cual = DbtModelNode {
            name: "model".to_owned(),
            database: "db".to_owned(),
            schema: "schema".to_owned(),
            materialized_as: AssetType::DBTable,
            ..Default::default()
        }
        .cual();

        assert_eq!(
            result_cual,
            Cual::new("snowflake://db/schema/model".to_owned())
        );
    }

    #[test]
    fn proper_source_node_yields_cual() {
        let result_cual = DbtSourceNode {
            name: "model".to_owned(),
            database: "db".to_owned(),
            schema: "schema".to_owned(),
            quoting: QuotingConfig::default(),
        }
        .cual();

        assert_eq!(
            result_cual,
            Cual::new("snowflake://db/schema/model".to_owned())
        );
    }

    #[test]
    fn no_quoting_config_yields_no_quotes() {
        let mut source_node = DbtSourceNode {
            name: "model".to_owned(),
            database: "db".to_owned(),
            schema: "schema".to_owned(),
            quoting: QuotingConfig {
                database: true,
                schema: false,
                identifier: false,
            },
        };

        // No quoting
        source_node.quoting = Default::default();
        let result_cual = source_node.cual();
        assert_eq!(
            result_cual,
            Cual::new("snowflake://db/schema/model".to_owned())
        );
    }

    #[test]
    fn db_quoting_config_results_in_quotes() {
        let mut source_node = DbtSourceNode {
            name: "model".to_owned(),
            database: "db".to_owned(),
            schema: "schema".to_owned(),
            quoting: QuotingConfig {
                database: true,
                schema: false,
                identifier: false,
            },
        };
        // Just db
        source_node.quoting = QuotingConfig {
            database: true,
            ..Default::default()
        };
        let result_cual = source_node.cual();
        assert_eq!(
            result_cual,
            Cual::new("snowflake://%22db%22/schema/model".to_owned())
        );
    }

    #[test]
    fn schema_quoting_config_results_in_quotes() {
        let mut source_node = DbtSourceNode {
            name: "model".to_owned(),
            database: "db".to_owned(),
            schema: "schema".to_owned(),
            quoting: QuotingConfig {
                database: true,
                schema: false,
                identifier: false,
            },
        };
        // Just schema
        source_node.quoting = QuotingConfig {
            schema: true,
            ..Default::default()
        };
        let result_cual = source_node.cual();
        assert_eq!(
            result_cual,
            Cual::new("snowflake://db/%22schema%22/model".to_owned())
        );
    }

    #[test]
    fn identifier_quoting_config_results_in_quotes() {
        let mut source_node = DbtSourceNode {
            name: "model".to_owned(),
            database: "db".to_owned(),
            schema: "schema".to_owned(),
            quoting: QuotingConfig {
                database: true,
                schema: false,
                identifier: false,
            },
        };
        // Just identifier
        source_node.quoting = QuotingConfig {
            identifier: true,
            ..Default::default()
        };
        let result_cual = source_node.cual();
        assert_eq!(
            result_cual,
            Cual::new("snowflake://db/schema/%22model%22".to_owned())
        );
    }

    #[test]
    fn db_schema_quoting_config_results_in_quotes() {
        let mut source_node = DbtSourceNode {
            name: "model".to_owned(),
            database: "db".to_owned(),
            schema: "schema".to_owned(),
            quoting: QuotingConfig {
                database: true,
                schema: false,
                identifier: false,
            },
        };
        // db and schema
        source_node.quoting = QuotingConfig {
            database: true,
            schema: true,
            ..Default::default()
        };
        let result_cual = source_node.cual();
        assert_eq!(
            result_cual,
            Cual::new("snowflake://%22db%22/%22schema%22/model".to_owned())
        );
    }

    #[test]
    fn db_schema_identifier_quoting_config_results_in_quotes() {
        let mut source_node = DbtSourceNode {
            name: "model".to_owned(),
            database: "db".to_owned(),
            schema: "schema".to_owned(),
            quoting: QuotingConfig {
                database: true,
                schema: false,
                identifier: false,
            },
        };
        // db and schema and identifier
        source_node.quoting = QuotingConfig {
            database: true,
            schema: true,
            identifier: true,
        };
        let result_cual = source_node.cual();
        assert_eq!(
            result_cual,
            Cual::new("snowflake://%22db%22/%22schema%22/%22model%22".to_owned())
        );
    }

    #[test]
    #[should_panic]
    fn unexpected_asset_type_panics() {
        DbtModelNode {
            materialized_as: AssetType::DBWarehouse,
            ..Default::default()
        }
        .cual();
    }
}
