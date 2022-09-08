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
            "table" => Ok(Some(AssetType::DBTable)),
            // TODO make this use the materialized_as field instead
            "model" => Ok(Some(AssetType::DBTable)),
            // TODO figure out what we want to do with seeds and tests
            "seed" | "test" => Ok(None),
            x => {
                println!("unexpected asset type {:?}", x);
                bail!("unexpected asset type");
            }
        }
    }
}
