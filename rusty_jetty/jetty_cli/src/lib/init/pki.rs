

use anyhow::Result;
use openssl::{pkey::PKey, rsa::Rsa};
use sha2::{Digest, Sha256};

/// Simple representation of a public/private key pair and
/// the public SHA256 fingerprint in the Snowflake format.
/// See https://jettylabs.tiny.us/snow-keypair-auth for more.
pub(crate) struct KeyPair {
    public: String,
    private: String,
    fingerprint: String,
}

impl KeyPair {
    /// Get the public key minus header/footer information. Just the key.
    pub(crate) fn public_inner(&self) -> String {
        let lines = self.public.lines().collect::<Vec<_>>();
        let len = lines.len();
        lines[1..len - 1].iter().cloned().collect::<String>()
    }

    pub(crate) fn fingerprint(&self) -> String {
        self.fingerprint.to_owned()
    }

    pub(crate) fn private_key(&self) -> String {
        self.private.to_owned()
    }
}

/// Create a local keypair with a corresponding public key fingerprint.
pub(crate) fn create_keypair() -> Result<KeyPair> {
    let rsa = PKey::from_rsa(Rsa::generate(2048)?)?;
    // Snowflake (and JWT creation) only accept PKCS8.
    let private = rsa.private_key_to_pem_pkcs8().map(String::from_utf8)??;
    let public = rsa.public_key_to_pem()?;
    // Fingerprint must be generated from der format.
    let public_der = rsa.public_key_to_der()?;
    let digest = Sha256::digest(public_der).to_vec();
    let fingerprint = format!("SHA256:{}", base64::encode(digest));
    Ok(KeyPair {
        private,
        fingerprint,
        public: String::from_utf8(public)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_keypair_works() -> Result<()> {
        let KeyPair {
            private,
            public,
            fingerprint: fp,
        } = create_keypair()?;
        assert!(private.starts_with("-----BEGIN PRIVATE KEY-----"));
        assert!(public.starts_with("-----BEGIN PUBLIC KEY-----"));
        assert!(fp.starts_with("SHA256:"));
        Ok(())
    }
}
