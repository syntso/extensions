use std::env;
use std::fs;
use std::io::Cursor;
use std::path::Path;

use minisign::{PublicKey, PublicKeyBox, SecretKey, SecretKeyBox};

use crate::error::RegistryError;

const SECRET_VARIABLE: &str = "MINISIGN_SECRET_KEY";
const PASSWORD_VARIABLE: &str = "MINISIGN_PASSWORD";
const UNTRUSTED: &str = "signature from the Synt extension registry";

pub struct Signer {
    secret: SecretKey,
    public: PublicKey,
}

impl Signer {
    pub fn from_environment(public_key: &Path) -> Result<Self, RegistryError> {
        let secret = env::var(SECRET_VARIABLE)
            .map_err(|_| RegistryError::MissingVariable(SECRET_VARIABLE))?;
        let password = env::var(PASSWORD_VARIABLE)
            .ok()
            .filter(|password| !password.is_empty());

        let public = fs::read_to_string(public_key).map_err(|source| RegistryError::Read {
            path: public_key.to_owned(),
            source,
        })?;

        Self::new(&secret, password, &public)
    }

    pub fn new(
        secret: &str,
        password: Option<String>,
        public: &str,
    ) -> Result<Self, RegistryError> {
        let secret_box = SecretKeyBox::from_string(secret.trim())?;

        let secret = match password {
            Some(password) => secret_box.into_secret_key(Some(password))?,
            None => secret_box.into_unencrypted_secret_key()?,
        };

        let public = PublicKeyBox::from_string(public.trim())?.into_public_key()?;

        Ok(Self { secret, public })
    }

    pub fn sign(&self, data: &[u8], comment: &str) -> Result<String, RegistryError> {
        let signature = minisign::sign(
            Some(&self.public),
            &self.secret,
            Cursor::new(data),
            Some(comment),
            Some(UNTRUSTED),
        )?;

        minisign::verify(
            &self.public,
            &signature,
            Cursor::new(data),
            true,
            false,
            false,
        )?;

        Ok(signature.into_string())
    }
}

#[cfg(test)]
mod tests {
    use minisign::{KeyPair, SignatureBox};

    use super::*;

    fn boxes() -> (String, String) {
        let pair = KeyPair::generate_unencrypted_keypair().expect("key pair");
        let secret = pair.sk.to_box(None).expect("secret box").into_string();
        let public = pair.pk.to_box().expect("public box").into_string();

        (secret, public)
    }

    #[test]
    fn signs_so_the_public_key_verifies() {
        let (secret, public) = boxes();
        let signer = Signer::new(&secret, None, &public).expect("signer");

        let signature = signer
            .sign(b"component", "synt extension uptime 0.1.0")
            .expect("signed");

        let signature = SignatureBox::from_string(&signature).expect("signature box");
        let public = PublicKeyBox::from_string(&public).and_then(PublicKeyBox::into_public_key);
        let public = public.expect("public key");

        assert!(
            minisign::verify(
                &public,
                &signature,
                Cursor::new(b"component"),
                true,
                false,
                false
            )
            .is_ok()
        );
        assert!(
            minisign::verify(
                &public,
                &signature,
                Cursor::new(b"tampered"),
                true,
                false,
                false
            )
            .is_err()
        );
        assert_eq!(
            signature.trusted_comment().expect("comment"),
            "synt extension uptime 0.1.0"
        );
    }

    #[test]
    fn refuses_a_public_key_from_another_pair() {
        let (secret, _) = boxes();
        let (_, other) = boxes();
        let signer = Signer::new(&secret, None, &other).expect("signer");

        assert!(signer.sign(b"component", "comment").is_err());
    }
}
