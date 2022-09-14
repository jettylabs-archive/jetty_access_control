use jetty_core::cual::{Cual, Cualable};

use crate::{Database, Schema, Table, View};

const NAMESPACE: &str = "snowflake";

impl Cualable for Table {
    /// Get the CUAL that points to this table or view.
    fn cual(&self) -> Cual {
        Cual {
            uri: format!(
                "{}://{}/{}/{}",
                NAMESPACE, self.database_name, self.schema_name, self.name
            ),
        }
    }
}

impl Cualable for View {
    /// Get the CUAL that points to this table or view.
    fn cual(&self) -> Cual {
        Cual {
            uri: format!(
                "{}://{}/{}/{}",
                NAMESPACE, self.database_name, self.schema_name, self.name
            ),
        }
    }
}

impl Cualable for Schema {
    /// Get the CUAL that points to this schema.
    fn cual(&self) -> Cual {
        Cual {
            uri: format!("{}://{}/{}", NAMESPACE, self.database_name, self.name),
        }
    }
}

impl Cualable for Database {
    /// Get the CUAL that points to this database.
    fn cual(&self) -> Cual {
        Cual {
            uri: format!("{}://{}", NAMESPACE, self.name),
        }
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
