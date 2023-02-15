use std::sync::Once;

use anyhow::{bail, Result};

use jetty_core::cual::Cualable;
// Reexport for convenience.
pub use jetty_core::cual::Cual;

use crate::{escape_snowflake_quotes, Database, Object, Schema, SnowflakeAsset};

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
            "{}://{}/{}?type=DATABASE",
            "snowflake",
            get_cual_account_name().expect("couldn't get CUAL account name"),
            urlencoding::encode(&$db)
        ))
    };
    ($db:expr, $schema:expr) => {
        Cual::new(&format!(
            "{}://{}/{}/{}?type=SCHEMA",
            "snowflake",
            get_cual_account_name().expect("couldn't get CUAL account name"),
            urlencoding::encode(&$db),
            urlencoding::encode(&$schema)
        ))
    };
    ($db:expr, $schema:expr, $table:expr, $asset_type:expr) => {
        Cual::new(&format!(
            "{}://{}/{}/{}/{}?type={}",
            "snowflake",
            get_cual_account_name().expect("couldn't get CUAL account name"),
            urlencoding::encode(&$db),
            urlencoding::encode(&$schema),
            urlencoding::encode(&$table),
            &$asset_type
        ))
    };
}

pub(crate) use cual;

pub(crate) fn cual_from_snowflake_obj_name(name: &str, asset_type: &str) -> Result<Cual> {
    let parts: Vec<_> = name
        .split('.')
        .map(|p| crate::strip_snowflake_quotes(p.to_owned(), true))
        .collect();

    if let (Some(db), Some(schema), Some(obj_name)) = (parts.get(0), parts.get(1), parts.get(2)) {
        Ok(cual!(db, schema, obj_name, asset_type))
    } else if let (Some(db), Some(schema)) = (parts.get(0), parts.get(1)) {
        Ok(cual!(db, schema))
    } else if let Some(db) = parts.get(0) {
        Ok(cual!(db))
    } else {
        bail!("name {} was not fully qualified", name)
    }
}

/// Given snowlake name parts, get a Cual
pub(crate) fn cual_from_snowflake_obj_name_parts(
    name: &str,
    db_name: &str,
    schema_name: &str,
    asset_type: &str,
) -> Result<Cual> {
    match asset_type {
        "DATABASE" => return Ok(cual!(escape_snowflake_quotes(name))),
        "SCHEMA" => return Ok(cual!(escape_snowflake_quotes(db_name), escape_snowflake_quotes(name))),
        "TABLE" | "VIEW" => return Ok(cual!(escape_snowflake_quotes(db_name), escape_snowflake_quotes(schema_name), escape_snowflake_quotes(name), asset_type)),
        _ => bail!("Unable to build cual for: db: {db_name}, schema: {schema_name:?}, name: {name}, type: {asset_type}")
    }
}

impl Cualable for Object {
    /// Get the CUAL that points to this table or view.
    fn cual(&self) -> Cual {
        cual!(
            escape_snowflake_quotes(&self.database_name),
            escape_snowflake_quotes(&self.schema_name),
            escape_snowflake_quotes(&self.name),
            self.kind.to_string()
        )
    }
}

impl Cualable for Schema {
    /// Get the CUAL that points to this schema.
    fn cual(&self) -> Cual {
        cual!(
            escape_snowflake_quotes(&self.database_name),
            escape_snowflake_quotes(&self.name)
        )
    }
}

impl Cualable for Database {
    /// Get the CUAL that points to this database.
    fn cual(&self) -> Cual {
        cual!(escape_snowflake_quotes(&self.name))
    }
}

pub(crate) fn cual_to_snowflake_asset(cual: &Cual) -> SnowflakeAsset {
    let path = cual.asset_path().components().to_owned();
    let asset_type = cual.asset_type().unwrap();
    let fqn = path
        .into_iter()
        .map(|segment| format!("\"{}\"", urlencoding::decode(segment.as_str()).unwrap()))
        .collect::<Vec<_>>()
        .join(".");
    match asset_type.to_string().as_str() {
        "TABLE" => SnowflakeAsset::Table(fqn),
        "VIEW" => SnowflakeAsset::View(fqn),
        "SCHEMA" => SnowflakeAsset::Schema(fqn),
        "DATABASE" => SnowflakeAsset::Database(fqn),
        _ => panic!("illegal snowflake asset type: {asset_type:?}"),
    }
}

#[cfg(test)]
mod tests {
    use crate::entry_types::ObjectKind;

    use super::*;

    #[test]
    fn test_cual_from_name() -> Result<()> {
        set_cual_account_name("account");
        let c =
            cual_from_snowflake_obj_name("SNOWFLAKE_SAMPLE_DATA.TPCDS_SF10TCL.WEB_PAGE", "TABLE")?;
        assert_eq!(
            c.uri(),
            "snowflake://account.snowflakecomputing.com/SNOWFLAKE_SAMPLE_DATA/TPCDS_SF10TCL/WEB_PAGE?type=TABLE".to_owned()
        );
        Ok(())
    }

    #[test]
    fn table_cual_constructs_properly() {
        set_cual_account_name("account");
        let cual = Object {
            name: "my_table".to_owned(),
            schema_name: "schema".to_owned(),
            database_name: "database".to_owned(),
            kind: ObjectKind::Table,
        }
        .cual();
        assert_eq!(
            cual,
            Cual::new(
                "snowflake://account.snowflakecomputing.com/database/schema/my_table?type=TABLE"
            )
        )
    }

    #[test]
    fn view_cual_constructs_properly() {
        set_cual_account_name("account");
        let cual = Object {
            name: "my_table".to_owned(),
            schema_name: "schema".to_owned(),
            database_name: "database".to_owned(),
            kind: ObjectKind::View,
        }
        .cual();
        assert_eq!(
            cual,
            Cual::new(
                "snowflake://account.snowflakecomputing.com/database/schema/my_table?type=VIEW"
            )
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
            Cual::new("snowflake://account.snowflakecomputing.com/database/my_schema?type=SCHEMA")
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
            Cual::new("snowflake://account.snowflakecomputing.com/my_db?type=DATABASE")
        )
    }

    #[test]
    fn cual_to_snowflake_asset_works_table() -> Result<()> {
        assert_eq!(
            cual_to_snowflake_asset(&Cual::new(
                "snowflake://account.snowflakecomputing.com/database/schema/my_table?type=TABLE",
            )),
            SnowflakeAsset::Table("\"database\".\"schema\".\"my_table\"".to_owned())
        );
        Ok(())
    }

    #[test]
    fn cual_to_snowflake_asset_works_db() -> Result<()> {
        assert_eq!(
            cual_to_snowflake_asset(&Cual::new(
                "snowflake://account.snowflakecomputing.com/databasE?type=DATABASE",
            )),
            SnowflakeAsset::Database("\"databasE\"".to_owned())
        );
        Ok(())
    }
}
