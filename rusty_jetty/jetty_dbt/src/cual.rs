use std::sync::Once;

use anyhow::{bail, Result};
// Convenience import
pub(crate) use jetty_core::cual::{Cual, Cualable};

use crate::manifest::node::NamePartable;

static mut CUAL_ACCOUNT_NAME: String = String::new();
static INIT_CUAL_ACCOUNT_NAME: Once = Once::new();

// Accessing a `static mut` is unsafe much of the time, but if we do so
// in a synchronized fashion (e.g., write once or read all) then we're
// good to go!
//
// This function will only set the string once, and will
// otherwise always effectively be a no-op.
pub(crate) fn set_cual_account_name(account_name: &str) {
    unsafe {
        INIT_CUAL_ACCOUNT_NAME.call_once(|| {
            CUAL_ACCOUNT_NAME = format!("{}.snowflakecomputing.com", account_name.to_lowercase())
        });
    }
}

pub(crate) fn get_cual_account_name<'a>() -> Result<&'a str> {
    if INIT_CUAL_ACCOUNT_NAME.is_completed() {
        // CUAL_PREFIX is set by a Once and is safe to use after initialization.
        unsafe { Ok(&CUAL_ACCOUNT_NAME) }
    } else {
        bail!("cual prefix was not yet set")
    }
}

/// Create a CUAL for a set of identifiers.
#[macro_export]
macro_rules! cual {
    ($db:expr) => {
        Cual::new(&format!("{}://{}", "snowflake", urlencoding::encode(&$db)))
    };
    ($db:expr, $schema:expr) => {
        Cual::new(&format!(
            "{}://{}/{}/{}",
            "snowflake",
            get_cual_account_name().expect("couldn't get CUAL account name"),
            urlencoding::encode(&$db),
            urlencoding::encode(&$schema)
        ))
    };
    ($db:expr, $schema:expr, $table:expr) => {
        Cual::new(&format!(
            "{}://{}/{}/{}/{}",
            "snowflake",
            get_cual_account_name().expect("couldn't get CUAL account name"),
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

    use crate::{
        consts::TABLE,
        manifest::node::{DbtModelNode, DbtSourceNode},
    };

    use super::*;

    #[test]
    fn macro_works() {
        let c = cual!("my_db").uri();
        assert_eq!(c, "snowflake://my_db")
    }

    #[test]
    fn proper_model_node_yields_cual() {
        set_cual_account_name("account");
        let result_cual = (&DbtModelNode {
            name: "db.schema.model".to_owned(),
            materialized_as: AssetType(TABLE.to_owned()),
            ..Default::default()
        } as &dyn NamePartable)
            .cual();

        assert_eq!(
            result_cual,
            Cual::new("snowflake://account.snowflakecomputing.com/DB/SCHEMA/MODEL")
        );
    }

    #[test]
    fn no_quoting_config_yields_no_quotes() {
        set_cual_account_name("account");
        let source_node = DbtSourceNode {
            name: r#"db.schema.model"#.to_owned(),
        };

        // No quoting
        let result_cual = (&source_node as &dyn NamePartable).cual();
        assert_eq!(
            result_cual,
            Cual::new("snowflake://account.snowflakecomputing.com/DB/SCHEMA/MODEL")
        );
    }

    #[test]
    fn db_quoting_config_results_in_quotes() {
        set_cual_account_name("account");
        let source_node = DbtSourceNode {
            name: r#"\"db\".schema.model"#.to_owned(),
        };
        // Just db
        let result_cual = (&source_node as &dyn NamePartable).cual();
        dbg!(&result_cual);
        assert_eq!(
            result_cual,
            Cual::new("snowflake://account.snowflakecomputing.com/db/SCHEMA/MODEL")
        );
    }

    #[test]
    fn schema_quoting_config_results_in_quotes() {
        set_cual_account_name("account");
        let source_node = DbtSourceNode {
            name: r#"db.\"schema\".model"#.to_owned(),
        };
        // Just schema
        let result_cual = (&source_node as &dyn NamePartable).cual();
        assert_eq!(
            result_cual,
            Cual::new("snowflake://account.snowflakecomputing.com/DB/schema/MODEL")
        );
    }

    #[test]
    fn identifier_quoting_config_results_in_quotes() {
        set_cual_account_name("account");
        let source_node = DbtSourceNode {
            name: r#"db.schema.\"model\""#.to_owned(),
        };
        // Just identifier
        let result_cual = (&source_node as &dyn NamePartable).cual();
        assert_eq!(
            result_cual,
            Cual::new("snowflake://account.snowflakecomputing.com/DB/SCHEMA/model")
        );
    }

    #[test]
    fn db_schema_quoting_config_results_in_quotes() {
        set_cual_account_name("account");
        let source_node = DbtSourceNode {
            name: r#"\"db\".\"schema\".model"#.to_owned(),
        };
        // db and schema
        let result_cual = (&source_node as &dyn NamePartable).cual();
        assert_eq!(
            result_cual,
            Cual::new("snowflake://account.snowflakecomputing.com/db/schema/MODEL")
        );
    }

    #[test]
    fn db_schema_identifier_quoting_config_results_in_quotes() {
        set_cual_account_name("account");
        let source_node = DbtSourceNode {
            name: r#"\"db\".\"schema\".\"model\""#.to_owned(),
        };
        // db and schema and identifier
        let result_cual = (&source_node as &dyn NamePartable).cual();
        assert_eq!(
            result_cual,
            Cual::new("snowflake://account.snowflakecomputing.com/db/schema/model")
        );
    }

    #[test]
    fn db_identifier_quoting_config_results_in_quotes() {
        set_cual_account_name("account");
        let source_node = DbtSourceNode {
            name: r#"\"db\".schema.\"model\""#.to_owned(),
        };
        // db and schema and identifier
        let result_cual = (&source_node as &dyn NamePartable).cual();
        assert_eq!(
            result_cual,
            Cual::new("snowflake://account.snowflakecomputing.com/db/SCHEMA/model")
        );
    }

    #[test]
    fn schema_identifier_quoting_config_results_in_quotes() {
        set_cual_account_name("account");
        let source_node = DbtSourceNode {
            name: r#"db.\"schema\".\"model\""#.to_owned(),
        };
        // db and schema and identifier
        let result_cual = (&source_node as &dyn NamePartable).cual();
        assert_eq!(
            result_cual,
            Cual::new("snowflake://account.snowflakecomputing.com/DB/schema/model")
        );
    }

    /// Periods between quotes aren't currently supported.
    #[test]
    #[should_panic]
    fn periods_in_quotes_panics() {
        set_cual_account_name("account");
        let source_node = DbtSourceNode {
            name: r#"db.\"schema.schema2\".\"model\""#.to_owned(),
        };
        // db and schema and identifier
        let result_cual = (&source_node as &dyn NamePartable).cual();
        assert_eq!(result_cual, Cual::new("snowflake://DB/schema/model"));
    }
}
