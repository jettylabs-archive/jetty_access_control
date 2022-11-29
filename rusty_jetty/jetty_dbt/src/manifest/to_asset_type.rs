use anyhow::{bail, Result};

use jetty_core::connectors::AssetType;

use crate::consts::{TABLE, VIEW};

/// Convert a resouce_type manifest string to an asset type.
pub(crate) trait ToAssetType {
    fn try_to_asset_type(&self) -> Result<Option<AssetType>>;
}

impl ToAssetType for String {
    fn try_to_asset_type(&self) -> Result<Option<AssetType>> {
        match self.as_str() {
            "view" => Ok(Some(AssetType(VIEW.to_owned()))),
            // Seeds are materialized into tables
            // (https://docs.getdbt.com/docs/building-a-dbt-project/seeds#faqs)
            "table" | "seed" | "snapshot" => Ok(Some(AssetType(TABLE.to_owned()))),
            "model" => Ok(Some(AssetType(TABLE.to_owned()))),
            // We ignore tests  and analyses since they aren't materialized.
            "test" | "analysis" => Ok(None),
            // We ignore exposures for now and will possibly integrate them later.
            "exposure" => Ok(None),
            x => {
                bail!("unexpected asset type {:?}", x);
            }
        }
    }
}
