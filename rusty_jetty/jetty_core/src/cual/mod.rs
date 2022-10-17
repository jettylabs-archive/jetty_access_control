//! Connector Universal Asset Locator
//!
//! This identifier serves as a standard for cross-connector asset addressing.
//!

use serde::{Deserialize, Serialize};

/// Just a CUAL
///
/// CUAL structs should be constructed with `Cual::new()`, not raw.
/// This is wrong:
/// ```compile_fail
/// # use jetty_core::cual::Cual;
/// Cual{uri:"cual:://use/new/instead"}; // uri is private
/// ```
///
/// This is correct:
/// ```
/// # use jetty_core::cual::Cual;
/// # let cual_str = "jetty_connector://my/custom/cual".to_owned();
/// Cual::new(cual_str);
/// ```
#[derive(Debug, PartialEq, Eq, Clone, Default, Hash, Deserialize, Serialize)]
pub struct Cual {
    /// The underlying URI that points to the asset.
    uri: String,
}

impl Cual {
    /// Create a new wrapper for the given URI.
    pub fn new(uri: String) -> Self {
        Self { uri }
    }

    /// Accessor for the underlying URI. This function makes it so we can
    /// protect creation of the raw struct from happening (forcing people
    /// to use ::new()).
    #[inline(always)]
    pub fn uri(&self) -> String {
        self.uri.to_owned()
    }
}

impl ToString for Cual {
    fn to_string(&self) -> String {
        self.uri.to_owned()
    }
}
/// Common behavior for connectors to implement.
pub trait Cualable {
    /// Get the cual for the associated asset object.
    fn cual(&self) -> Cual;
}
