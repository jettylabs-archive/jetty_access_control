use std::{fs::File, io::Write, path::Path};

use anyhow::Result;
use openssl::{pkey::PKey, rsa::Rsa, symm::Cipher};
use sha2::{Digest, Sha256};

pub(crate) struct KeyPair {
    public: String,
    private: String,
    fingerprint: String,
}

impl KeyPair {
    pub(crate) fn save_to_files(&self, filepath: &Path) -> Result<()> {
        save_to_file(&self.private, &filepath.join("jetty_rsa.p8"))?;
        save_to_file(&self.public, &filepath.join("jetty_rsa.pub"))?;
        Ok(())
    }

    /// Get the public key minus header/footer information. Just the key.
    pub(crate) fn public_inner(&self) -> String {
        let lines = self.public.lines().collect::<Vec<_>>();
        let len = lines.len();
        lines[1..len - 1].into_iter().cloned().collect::<String>()
    }

    pub(crate) fn fingerprint(&self) -> String {
        self.fingerprint.to_owned()
    }

    pub(crate) fn private_key(&self) -> String {
        self.private.to_owned()
    }
}

fn save_to_file(contents: &str, filepath: &Path) -> Result<()> {
    let mut file = File::create(filepath)?;
    file.write_all(contents.as_bytes())?;
    Ok(())
}

pub(crate) fn create_keypair() -> Result<KeyPair> {
    let rsa = PKey::from_rsa(Rsa::generate(2048)?)?;
    // Snowflake (and JWT creation) only accept PKCS8.
    let private = rsa.private_key_to_pem_pkcs8().map(String::from_utf8)??;
    let public = rsa.public_key_to_pem()?;
    // Fingerprint must be generated from der format.
    let public_der = rsa.public_key_to_der()?;
    let digest = Sha256::digest(&public_der).to_vec();
    let fingerprint = format!("SHA256:{}", base64::encode(&digest));
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
        assert!(private.starts_with("-----BEGIN RSA PRIVATE KEY-----"));
        assert!(public.starts_with("-----BEGIN RSA PUBLIC KEY-----"));
        assert!(fp.starts_with("SHA256:"));
        Ok(())
    }
}
