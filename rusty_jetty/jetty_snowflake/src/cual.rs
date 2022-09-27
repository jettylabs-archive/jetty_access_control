use anyhow::{bail, Result};

use jetty_core::cual::Cualable;
// Reexport for convenience.
pub use jetty_core::cual::Cual;

use crate::{Database, Schema, Table, View};

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

pub(crate) fn cual_from_snowflake_obj_name(name: &str) -> Result<Cual> {
    let parts: Vec<_> = name.split('.').map(str::to_lowercase).collect();

    if let Some(db) = parts.get(0) {
        Ok(cual!(db))
    } else if let [db, schema] = &parts[0..1] {
        Ok(cual!(db, schema))
    } else if let [db, schema, obj_name] = &parts[0..2] {
        Ok(cual!(db, schema, obj_name))
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
    fn table_cual_constructs_properly() {
        let cual = Table {
            name: "my_table".to_owned(),
            schema_name: "schema".to_owned(),
            database_name: "database".to_owned(),
        }
        .cual();
        assert_eq!(
            cual,
            Cual::new("snowflake://database/schema/my_table".to_owned())
        )
    }

    #[test]
    fn view_cual_constructs_properly() {
        let cual = View {
            name: "my_table".to_owned(),
            schema_name: "schema".to_owned(),
            database_name: "database".to_owned(),
        }
        .cual();
        assert_eq!(
            cual,
            Cual::new("snowflake://database/schema/my_table".to_owned())
        )
    }

    #[test]
    fn schema_cual_constructs_properly() {
        let cual = Schema {
            name: "my_schema".to_owned(),
            database_name: "database".to_owned(),
        }
        .cual();
        assert_eq!(cual, Cual::new("snowflake://database/my_schema".to_owned()))
    }

    #[test]
    fn db_cual_constructs_properly() {
        let cual = Database {
            name: "my_db".to_owned(),
        }
        .cual();
        assert_eq!(cual, Cual::new("snowflake://my_db".to_owned()))
    }
}
