//! Connector Universal Asset Locator
//!
//! This identifier serves as a standard for cross-connector asset addressing.
//!

/// Just a CUAL
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Cual {
    /// The underlying URI that points to the asset.
    pub uri: String,
}

impl Cual {
    /// Create a new wrapper for the given URI.
    pub fn new(uri: String) -> Self {
        Self {
            uri: uri.to_lowercase(),
        }
    }
}

/// Common behavior for connectors to implement.
pub trait Cualable {
    /// Get the cual for the associated asset object.
    fn cual(&self) -> Cual;
}
