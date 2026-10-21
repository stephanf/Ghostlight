//! Local single-user identity, available only through the explicit --local host.
use anyhow::{Context, ensure};
use axum::http::{HeaderMap, header};
use sha2::{Digest, Sha256};
use std::{fs::{self, OpenOptions}, io::Write, net::SocketAddr, path::Path};

#[derive(Clone)]
pub(crate) struct LocalAccess {
    token_digest: [u8; 32],
}

pub(crate) fn local_bind(value: Option<&str>) -> anyhow::Result<SocketAddr> {
    let bind: SocketAddr = value.unwrap_or("127.0.0.1:8831").parse()
        .context("GHOSTLIGHT_LOCAL_BIND must be a socket address")?;
    ensure!(bind.ip().is_loopback(), "local mode must bind to loopback; use SSH port forwarding for remote access");
    Ok(bind)
}

/// Create once with restrictive permissions; never overwrite an existing secret.
pub(crate) fn create_secret(path: &Path) -> anyhow::Result<()> {
    let parent = path.parent().context("secret has no parent directory")?;
    fs::create_dir_all(parent)?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)] {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    match options.open(path) {
        Ok(mut file) => {
            file.write_all(&rand::random::<[u8; 32]>())?;
            file.sync_all()?;
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let metadata = fs::symlink_metadata(path)?;
            ensure!(metadata.is_file() && !metadata.file_type().is_symlink(), "local secret must be a regular file");
            #[cfg(unix)] {
                use std::os::unix::fs::PermissionsExt;
                ensure!(metadata.permissions().mode() & 0o077 == 0, "local secret must not be accessible to other users");
            }
        }
        Err(error) => return Err(error.into()),
    }
    ensure!(fs::metadata(path)?.len() == 32, "local secret must contain exactly 32 bytes");
    Ok(())
}

impl LocalAccess {
    pub(crate) fn open(root: &Path) -> anyhow::Result<Self> {
        let path = root.join("secrets/local-api.token");
        create_secret(&path)?;
        let bytes = fs::read(path)?;
        // Header representation is hex of the raw token file; never printed or logged.
        let token: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
        Ok(Self { token_digest: Sha256::digest(token.as_bytes()).into() })
    }

    pub(crate) fn authorizes(&self, headers: &HeaderMap) -> Option<()> {
        let token = headers.get(header::AUTHORIZATION)?.to_str().ok()?.strip_prefix("Bearer ")?;
        let supplied: [u8; 32] = Sha256::digest(token.as_bytes()).into();
        let difference = self.token_digest.iter().zip(supplied).fold(0u8, |diff, (a, b)| diff | (a ^ b));
        if difference != 0 { return None; }
        Some(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_host_refuses_network_exposure() {
        assert!(local_bind(None).unwrap().ip().is_loopback());
        assert!(local_bind(Some("[::1]:8831")).is_ok());
        assert!(local_bind(Some("0.0.0.0:8831")).is_err());
        assert!(local_bind(Some("192.168.1.10:8831")).is_err());
    }

    #[test]
    fn local_identity_requires_token_and_survives_reopen() {
        let root = tempfile::tempdir().unwrap();
        let access = LocalAccess::open(root.path()).unwrap();
        let bytes = fs::read(root.path().join("secrets/local-api.token")).unwrap();
        let token: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        let mut headers = HeaderMap::new();
        assert!(access.authorizes(&headers).is_none());
        headers.insert(header::COOKIE, "ghostlight_session=anything".parse().unwrap());
        assert!(access.authorizes(&headers).is_none());
        headers.insert(header::AUTHORIZATION, "Bearer wrong".parse().unwrap());
        assert!(access.authorizes(&headers).is_none());
        headers.insert(header::AUTHORIZATION, format!("Bearer {token}").parse().unwrap());
        assert!(access.authorizes(&headers).is_some());
        let reopened = LocalAccess::open(root.path()).unwrap();
        assert!(reopened.authorizes(&headers).is_some());
    }

    #[test]
    fn existing_invalid_secret_is_refused_without_overwriting() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("bad");
        fs::write(&path, "invalid").unwrap();
        assert!(create_secret(&path).is_err());
        assert_eq!(fs::read_to_string(path).unwrap(), "invalid");
    }
}
