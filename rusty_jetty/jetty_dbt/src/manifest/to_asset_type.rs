use anyhow::{bail, Result};

use jetty_core::connectors::AssetType;

/// Convert a string to a
pub(crate) trait ToAssetType {
    fn try_to_asset_type(&self) -> Result<Option<AssetType>>;
}

impl ToAssetType for String {
    fn try_to_asset_type(&self) -> Result<Option<AssetType>> {
        match self.as_str() {
            "view" => Ok(Some(AssetType::DBView)),
            // Seeds are materialized into tables
            // (https://docs.getdbt.com/docs/building-a-dbt-project/seeds#faqs)
            "table" | "seed" => Ok(Some(AssetType::DBTable)),
            "model" => Ok(Some(AssetType::DBTable)),
            // We ignore tests since they aren't materialized.
            "test" => Ok(None),
            x => {
                println!("unexpected asset type {:?}", x);
                bail!("unexpected asset type");
            }
        }
    }
}
