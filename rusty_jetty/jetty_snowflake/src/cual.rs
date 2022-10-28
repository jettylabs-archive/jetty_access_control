use std::sync::Once;

use anyhow::{bail, Result};

use jetty_core::cual::Cualable;
// Reexport for convenience.
pub use jetty_core::cual::Cual;

use crate::{Database, Object, Schema, Table, View};

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

pub(crate) fn get_cual_prefix() -> Result<String> {
    if INIT_CUAL_ACCOUNT_NAME.is_completed() {
        // CUAL_PREFIX is set by a Once and is safe to use after initialization.
        Ok(format!(
            "{}://{}",
            "snowflake",
            get_cual_account_name().expect("couldn't get CUAL account name")
        ))
    } else {
        bail!("cual prefix was not yet set")
    }
}

macro_rules! cual {
    ($db:expr) => {
        Cual::new(&format!(
            "{}://{}/{}",
            "snowflake",
            get_cual_account_name().expect("couldn't get CUAL account name"),
            urlencoding::encode(&$db)
        ))
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

pub(crate) fn cual_from_snowflake_obj_name(name: &str) -> Result<Cual> {
    let parts: Vec<_> = name
        .split('.')
        .map(|p| {
            if p.starts_with("\"\"\"") {
                p.replace("\"\"\"", "\"")
            } else if p.starts_with('"') {
                // Remove the quotes and return the contained part as-is.
                p.trim_matches('"').to_owned()
            } else {
                // Not quoted – we can just capitalize it (only for
                // Snowflake).
                p.to_uppercase()
            }
        })
        .collect();

    if let (Some(db), Some(schema), Some(obj_name)) = (parts.get(0), parts.get(1), parts.get(2)) {
        Ok(cual!(db, schema, obj_name))
    } else if let (Some(db), Some(schema)) = (parts.get(0), parts.get(1)) {
        Ok(cual!(db, schema))
    } else if let Some(db) = parts.get(0) {
        Ok(cual!(db))
    } else {
        bail!("name {} was not fully qualified", name)
    }
}

impl Cualable for Table {
    /// Get the CUAL that points to this table or view.
    fn cual(&self) -> Cual {
        cual!(self.database_name, self.schema_name, self.name)
    }
}

impl Cualable for Object {
    /// Get the CUAL that points to this table or view.
    fn cual(&self) -> Cual {
        cual!(self.database_name, self.schema_name, self.name)
    }
}

impl Cualable for View {
    /// Get the CUAL that points to this table or view.
    fn cual(&self) -> Cual {
        cual!(self.database_name, self.schema_name, self.name)
    }
}

impl Cualable for Schema {
    /// Get the CUAL that points to this schema.
    fn cual(&self) -> Cual {
        cual!(self.database_name, self.name)
    }
}

impl Cualable for Database {
    /// Get the CUAL that points to this database.
    fn cual(&self) -> Cual {
        cual!(self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cual_from_name() -> Result<()> {
        set_cual_account_name("account");
        let c = cual_from_snowflake_obj_name("SNOWFLAKE_SAMPLE_DATA.TPCDS_SF10TCL.WEB_PAGE")?;
        assert_eq!(
            c.uri(),
            "snowflake://account.snowflakecomputing.com/SNOWFLAKE_SAMPLE_DATA/TPCDS_SF10TCL/WEB_PAGE".to_owned()
        );
        Ok(())
    }

    #[test]
    fn table_cual_constructs_properly() {
        set_cual_account_name("account");
        let cual = Table {
            name: "my_table".to_owned(),
            schema_name: "schema".to_owned(),
            database_name: "database".to_owned(),
        }
        .cual();
        assert_eq!(
            cual,
            Cual::new("snowflake://account.snowflakecomputing.com/database/schema/my_table")
        )
    }

    #[test]
    fn view_cual_constructs_properly() {
        set_cual_account_name("account");
        let cual = View {
            name: "my_table".to_owned(),
            schema_name: "schema".to_owned(),
            database_name: "database".to_owned(),
        }
        .cual();
        assert_eq!(
            cual,
            Cual::new("snowflake://account.snowflakecomputing.com/database/schema/my_table")
        )
    }

    #[test]
    fn schema_cual_constructs_properly() {
        set_cual_account_name("account");
        let cual = Schema {
            name: "my_schema".to_owned(),
            database_name: "database".to_owned(),
        }
        .cual();
        assert_eq!(
            cual,
            Cual::new("snowflake://account.snowflakecomputing.com/database/my_schema")
        )
    }

    #[test]
    fn db_cual_constructs_properly() {
        set_cual_account_name("account");
        let cual = Database {
            name: "my_db".to_owned(),
        }
        .cual();
        assert_eq!(
            cual,
            Cual::new("snowflake://account.snowflakecomputing.com/my_db")
        )
    }
}
