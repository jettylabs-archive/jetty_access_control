//! Connector Universal Asset Locator
//!
//! This identifier serves as a standard for cross-connector asset addressing.
//!

use anyhow::Context;
use serde::{Deserialize, Serialize};
use url::Url;

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
/// # let cual_str = "jettyconnector://my/custom/cual";
/// Cual::new(cual_str);
/// ```
#[derive(Debug, PartialEq, Eq, Clone, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct Cual {
    /// The underlying URI that points to the asset.
    uri: Url,
}

impl Cual {
    /// Create a new wrapper for the given URI.
    pub fn new(uri: &str) -> Self {
        Self {
            uri: Url::parse(uri).context("creating cual from uri").unwrap(),
        }
    }

    /// Accessor for the underlying URI. This function makes it so we can
    /// protect creation of the raw struct from happening (forcing people
    /// to use ::new()).
    #[inline(always)]
    pub fn uri(&self) -> String {
        self.uri.to_string()
    }

    /// Get the scheme for this CUAL.
    ///
    /// The scheme for `tableau`://path/to/asset?type=project would be `tableau`
    pub fn scheme(&self) -> &str {
        self.uri.scheme()
    }
}

impl ToString for Cual {
    fn to_string(&self) -> String {
        self.uri.to_string()
    }
}

impl Default for Cual {
    fn default() -> Self {
        Self {
            uri: Url::parse("jetty://default").unwrap(),
        }
    }
}
/// Common behavior for connectors to implement.
pub trait Cualable {
    /// Get the cual for the associated asset object.
    fn cual(&self) -> Cual;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cual_scheme_works() {
        let c = Cual::new("unicornz://sparkle/glitter/city?sunshine=yes");
        assert_eq!(c.scheme(), "unicornz");
    }

    #[test]
    fn test_default_cual_works() {
        let c = Cual::default();
        assert_eq!(c.uri(), "jetty://default");
        assert_eq!(c.scheme(), "jetty");
        assert_eq!(c.to_string(), "jetty://default".to_owned());
    }
}
