use std::{fs::File, io::Write, path::Path};

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
    /// Create a local keypair with a corresponding public key fingerprint.
    pub(crate) fn new() -> Result<KeyPair> {
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

    /// Load a keypair from the given filepaths.
    pub(crate) fn from_path(filepath: impl AsRef<Path>) -> Result<KeyPair> {
        todo!("Isaac will fix this")
    }

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

    /// Put both public and private keys in files with `name` in `dir`.
    ///
    /// `name` is the name of the file (`.p8` appended for private, `.pub`
    /// appended for public)
    ///
    /// `dir` is the directory where the files will be created.
    pub(crate) fn save_to_files(&self, filepath: impl AsRef<Path>) -> Result<()> {
        let pub_path = filepath.as_ref().to_path_buf().with_extension("p8");
        let priv_path = filepath.as_ref().to_path_buf().with_extension("pub");
        save_to_file(&self.public, &pub_path)?;
        save_to_file(&self.private, &priv_path)?;
        Ok(())
    }
}

fn save_to_file(contents: &str, filepath: impl AsRef<Path>) -> Result<()> {
    let mut file = File::create(filepath)?;
    file.write_all(contents.as_bytes())?;
    Ok(())
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
        } = KeyPair::new()?;
        assert!(private.starts_with("-----BEGIN PRIVATE KEY-----"));
        assert!(public.starts_with("-----BEGIN PUBLIC KEY-----"));
        assert!(fp.starts_with("SHA256:"));
        Ok(())
    }
}
