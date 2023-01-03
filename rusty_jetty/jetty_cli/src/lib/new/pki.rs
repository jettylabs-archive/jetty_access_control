use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use anyhow::Result;

use rsa::{
    pkcs8::{DecodePrivateKey, EncodePrivateKey, EncodePublicKey, LineEnding},
    RsaPrivateKey,
};
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
        let mut rng = rand::thread_rng();

        let bits = 2048;
        let private_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");

        key_pair_from_private_key(&private_key)
    }

    /// Load a keypair from the given filepaths.
    pub(crate) fn from_path(filepath: impl AsRef<Path>) -> Result<KeyPair> {
        let private_key_string = fs::read_to_string(filepath)?;
        let private_key = RsaPrivateKey::from_pkcs8_pem(private_key_string.as_str())?;
        key_pair_from_private_key(&private_key)
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
        let filepath = filepath.as_ref();

        // Create the parent directories if they don't exist
        if let Some(p) = filepath.parent() {
            fs::create_dir_all(p)?;
        }
        let priv_path = filepath.to_path_buf();
        let pub_path = filepath.to_path_buf().with_extension("pub");
        save_to_file(&self.public, pub_path)?;
        save_to_file(&self.private, priv_path)?;
        Ok(())
    }
}

fn save_to_file(contents: &str, filepath: impl AsRef<Path>) -> Result<()> {
    let mut file = File::create(filepath)?;
    file.write_all(contents.as_bytes())?;
    Ok(())
}

fn key_pair_from_private_key(private_key: &RsaPrivateKey) -> Result<KeyPair> {
    let private_as_p8 = private_key.to_pkcs8_pem(LineEnding::default())?.to_string();

    let public_key = private_key.to_public_key();
    let public_pem = public_key.to_public_key_pem(LineEnding::default())?;
    let public_der = public_key.to_public_key_der()?;

    let digest = Sha256::digest(public_der).to_vec();
    let fingerprint = format!("SHA256:{}", base64::encode(digest));

    Ok(KeyPair {
        private: private_as_p8,
        fingerprint,
        public: public_pem,
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
        } = KeyPair::new()?;
        assert!(private.starts_with("-----BEGIN PRIVATE KEY-----"));
        assert!(public.starts_with("-----BEGIN PUBLIC KEY-----"));
        assert!(fp.starts_with("SHA256:"));
        Ok(())
    }
}
