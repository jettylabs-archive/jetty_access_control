use jetty_core::{
    connectors::AssetType,
    cual::{Cual, Cualable},
};

use crate::manifest::node::{DbtModelNode, DbtNode, DbtSourceNode};

const SNOW_NAMESPACE: &str = "snowflake";
const DBT_NAMESPACE: &str = "dbt";

impl Cualable for DbtModelNode {
    fn cual(&self) -> Cual {
        // If the model is materialized as a warehouse object, it gets a
        // warehouse-specific CUAL.
        // Otherwise, it gets a dbt CUAL.
        match self.materialized_as {
            AssetType::DBTable | AssetType::DBView => Cual {
                uri: format!(
                    "{}://{}/{}/{}",
                    SNOW_NAMESPACE,
                    self.database.to_owned(),
                    self.schema.to_owned(),
                    self.name.to_owned()
                ),
            },
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
        Cual {
            uri: format!(
                "snowflake://{}/{}/{}",
                self.database, self.schema, self.name
            ),
        }
    }
}

mod test {
    use crate::manifest::node::DbtModelNode;

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
        }
        .cual();

        assert_eq!(
            result_cual,
            Cual::new("snowflake://db/schema/model".to_owned())
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
