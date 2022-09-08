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
