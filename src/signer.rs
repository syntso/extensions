use std::io::Cursor;

use data_encoding::BASE64;
use minisign::{PublicKey, PublicKeyBox, SecretKey};

use crate::RegistryError;

const UNTRUSTED_COMMENT: &str = "signed by the synt extensions registry";
const NO_PASSWORD: [u8; 2] = [0, 0];

pub struct Signer {
    public: PublicKey,
    secret: SecretKey,
}

impl Signer {
    pub fn new(secret_key: &str, registry_pub: &str) -> Result<Self, RegistryError> {
        let secret = Self::secret(secret_key)?;

        let public = PublicKey::from_secret_key(&secret)
            .map_err(|error| RegistryError::Key(error.to_string()))?;

        let trusted = PublicKeyBox::from_string(registry_pub.trim())
            .and_then(PublicKeyBox::into_public_key)
            .map_err(|error| RegistryError::Key(error.to_string()))?;

        if public.to_base64() != trusted.to_base64() {
            return Err(RegistryError::ForeignKey(trusted.to_base64()));
        }

        Ok(Self { public, secret })
    }

    pub fn sign(&self, bytes: &[u8], trusted_comment: &str) -> Result<String, RegistryError> {
        let signature = minisign::sign(
            Some(&self.public),
            &self.secret,
            Cursor::new(bytes),
            Some(trusted_comment),
            Some(UNTRUSTED_COMMENT),
        )
        .map_err(|error| RegistryError::Sign(error.to_string()))?;

        Ok(signature.to_string())
    }

    fn secret(text: &str) -> Result<SecretKey, RegistryError> {
        let line = text
            .lines()
            .nth(1)
            .ok_or_else(|| RegistryError::Key("not a minisign secret key file".to_owned()))?;

        let bytes = BASE64
            .decode(line.trim().as_bytes())
            .map_err(|error| RegistryError::Key(error.to_string()))?;

        if bytes.get(2..4) != Some(NO_PASSWORD.as_slice()) {
            return Err(RegistryError::Key(
                "it has a password; the registry signs with a key made by minisign -G -W"
                    .to_owned(),
            ));
        }

        SecretKey::from_bytes(&bytes).map_err(|error| RegistryError::Key(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use minisign::KeyPair;
    use minisign_verify::{PublicKey, Signature};

    use super::Signer;

    fn pair() -> (String, String) {
        let pair = KeyPair::generate_unencrypted_keypair().unwrap();
        let secret = pair.sk.to_box(None).unwrap().to_string();
        let public = pair.pk.to_box().unwrap().to_string();

        (secret, public)
    }

    #[test]
    fn signatures_check_out_the_way_the_core_checks_them() {
        let (secret, public) = pair();
        let signer = Signer::new(&secret, &public).unwrap();

        let text = signer
            .sign(b"index", r#"{"serial":1,"expires":2}"#)
            .unwrap();

        let key = PublicKey::decode(&public).unwrap();
        let signature = Signature::decode(&text).unwrap();
        key.verify(b"index", &signature, false).unwrap();
        assert_eq!(signature.trusted_comment(), r#"{"serial":1,"expires":2}"#);
    }

    #[test]
    fn reads_keys_of_the_minisign_command_which_leave_the_checksum_empty() {
        let (secret, public) = pair();
        let mut lines: Vec<String> = secret.lines().map(str::to_owned).collect();
        let mut bytes = data_encoding::BASE64.decode(lines[1].as_bytes()).unwrap();
        let length = bytes.len();
        bytes[length - 32..].fill(0);
        lines[1] = data_encoding::BASE64.encode(&bytes);

        let signer = Signer::new(&lines.join("\n"), &public).unwrap();

        assert!(signer.sign(b"index", "{}").is_ok());
    }

    #[test]
    fn a_key_with_a_password_is_refused() {
        let (secret, public) = pair();
        let mut lines: Vec<String> = secret.lines().map(str::to_owned).collect();
        let mut bytes = data_encoding::BASE64.decode(lines[1].as_bytes()).unwrap();
        bytes[2..4].copy_from_slice(b"Sc");
        lines[1] = data_encoding::BASE64.encode(&bytes);

        let refusal = Signer::new(&lines.join("\n"), &public)
            .err()
            .unwrap()
            .to_string();

        assert!(refusal.contains("password"), "{refusal}");
    }

    #[test]
    fn a_key_that_registry_pub_does_not_name_is_refused() {
        let (secret, _) = pair();
        let (_, other) = pair();

        let refusal = Signer::new(&secret, &other).err().unwrap().to_string();

        assert!(refusal.contains("registry.pub"), "{refusal}");
    }

    #[test]
    fn the_published_key_reads() {
        let (secret, _) = pair();

        let refusal = Signer::new(&secret, include_str!("../registry.pub"))
            .err()
            .unwrap();

        assert!(refusal.to_string().contains("does not belong"), "{refusal}");
    }
}
