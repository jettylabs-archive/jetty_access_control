//! Connector Universal Asset Locator
//!
//! This identifier serves as a standard for cross-connector asset addressing.
//!

/// Just a CUAL
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Cual {
    /// The underlying URI that points to the asset.
    uri: String,
}

impl Cual {
    /// Create a new wrapper for the given URI.
    ///
    /// CUALs are case-insensitive. So we coerce all cuals to lowercase
    /// to apply uniformity across connectors.
    pub fn new(uri: String) -> Self {
        Self {
            uri: uri.to_lowercase(),
        }
    }

    /// Accessor for the underlying URI. This function makes it so we can
    /// protect creation of the raw struct from happening (forcing people
    /// to use ::new()).
    #[inline(always)]
    pub fn uri(&self) -> String {
        self.uri.to_owned()
    }
}

/// Common behavior for connectors to implement.
pub trait Cualable {
    /// Get the cual for the associated asset object.
    fn cual(&self) -> Cual;
}
