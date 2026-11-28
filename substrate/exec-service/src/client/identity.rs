use std::{
    fs::{self, File},
    io::{self, Write as _},
    path::Path,
};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ed25519_dalek::SigningKey;
use prometheus_exec_contracts::{SignatureAlgorithm, VerificationKey};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentityFile {
    pub schema_version: String,
    pub sig_alg: SignatureAlgorithm,
    pub key_id: String,
    pub public_key: String,
    pub private_key: String,
}

pub struct LoadedIdentity {
    pub file: IdentityFile,
    pub signing_key: SigningKey,
    pub verification_key: VerificationKey,
}

pub fn create(path: &Path) -> Result<IdentityFile, Box<dyn std::error::Error + Send + Sync>> {
    let parent = parent_or_current(path);
    fs::create_dir_all(parent)?;
    let key = SigningKey::generate(&mut OsRng);
    let public = VerificationKey::ed25519(key.verifying_key().to_bytes());
    let identity = IdentityFile {
        schema_version: "1".into(),
        sig_alg: SignatureAlgorithm::Ed25519,
        key_id: public.key_id(),
        public_key: public.to_base64url(),
        private_key: URL_SAFE_NO_PAD.encode(key.to_bytes()),
    };
    let mut bytes = serde_json::to_vec_pretty(&identity)?;
    bytes.push(b'\n');
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    set_private_permissions(temporary.as_file())?;
    temporary.write_all(&bytes)?;
    temporary.as_file().sync_all()?;
    temporary.persist_noclobber(path)?;
    sync_directory(parent)?;
    Ok(identity)
}

pub fn load(path: &Path) -> Result<LoadedIdentity, Box<dyn std::error::Error + Send + Sync>> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "identity must be a regular non-symlink file: {}",
            path.display()
        )
        .into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(format!("identity must have mode 0600: {}", path.display()).into());
        }
    }
    let identity: IdentityFile = serde_json::from_slice(&fs::read(path)?)?;
    if identity.schema_version != "1" || identity.sig_alg != SignatureAlgorithm::Ed25519 {
        return Err("identity schema or signature algorithm is unsupported".into());
    }
    let private = URL_SAFE_NO_PAD.decode(&identity.private_key)?;
    let private: [u8; 32] = private
        .as_slice()
        .try_into()
        .map_err(|_| "identity private key must contain 32 bytes")?;
    let signing_key = SigningKey::from_bytes(&private);
    let verification_key = VerificationKey::ed25519(signing_key.verifying_key().to_bytes());
    if identity.public_key != verification_key.to_base64url()
        || identity.key_id != verification_key.key_id()
    {
        return Err("identity public key or key ID does not match its private key".into());
    }
    Ok(LoadedIdentity {
        file: identity,
        signing_key,
        verification_key,
    })
}

fn parent_or_current(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

#[cfg(unix)]
fn set_private_permissions(file: &File) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt as _;
    file.set_permissions(fs::Permissions::from_mode(0o600))
}

#[cfg(not(unix))]
fn set_private_permissions(_file: &File) -> io::Result<()> {
    Ok(())
}

fn sync_directory(path: &Path) -> io::Result<()> {
    File::open(path)?.sync_all()
}
