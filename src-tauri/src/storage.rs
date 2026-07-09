mod file_store {
    use keyring_core::api::{CredentialApi, CredentialPersistence, CredentialStoreApi};
    use keyring_core::{Credential, CredentialStore, Entry, Error, Result};
    use std::any::Any;
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Arc;

    fn credential_path(base: &std::path::Path, _service: &str, user: &str) -> PathBuf {
        let safe = |s: &str| s.replace(['/', '\\', '\0', ':'], "_");
        base.join("credentials").join(safe(user))
    }

    #[derive(Debug, Clone)]
    pub struct FileCredential {
        service: String,
        user: String,
        base: PathBuf,
    }

    impl CredentialApi for FileCredential {
        fn set_secret(&self, secret: &[u8]) -> Result<()> {
            #[cfg(unix)]
            use std::os::unix::fs::PermissionsExt;
            let path = credential_path(&self.base, &self.service, &self.user);
            let dir = path.parent().ok_or_else(|| {
                Error::PlatformFailure(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "credential path has no parent directory",
                )))
            })?;
            fs::create_dir_all(dir).map_err(|e| Error::PlatformFailure(Box::new(e)))?;
            #[cfg(unix)]
            fs::set_permissions(dir, fs::Permissions::from_mode(0o700))
                .map_err(|e| Error::PlatformFailure(Box::new(e)))?;
            fs::write(&path, secret).map_err(|e| Error::PlatformFailure(Box::new(e)))?;
            #[cfg(unix)]
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600))
                .map_err(|e| Error::PlatformFailure(Box::new(e)))?;
            Ok(())
        }

        fn get_secret(&self) -> Result<Vec<u8>> {
            let path = credential_path(&self.base, &self.service, &self.user);
            match fs::read(&path) {
                Ok(bytes) => Ok(bytes),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(Error::NoEntry),
                Err(e) => Err(Error::PlatformFailure(Box::new(e))),
            }
        }

        fn delete_credential(&self) -> Result<()> {
            let path = credential_path(&self.base, &self.service, &self.user);
            match fs::remove_file(&path) {
                Ok(()) => Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(Error::NoEntry),
                Err(e) => Err(Error::PlatformFailure(Box::new(e))),
            }
        }

        fn get_credential(&self) -> Result<Option<Arc<Credential>>> {
            Ok(None)
        }

        fn get_specifiers(&self) -> Option<(String, String)> {
            Some((self.service.clone(), self.user.clone()))
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[derive(Debug)]
    pub struct FileStore {
        base: PathBuf,
    }

    impl CredentialStoreApi for FileStore {
        fn vendor(&self) -> String {
            "open-grind file store".to_string()
        }

        fn id(&self) -> String {
            "file-store-v1".to_string()
        }

        fn build(
            &self,
            service: &str,
            user: &str,
            _modifiers: Option<&HashMap<&str, &str>>,
        ) -> Result<Entry> {
            let cred = Arc::new(FileCredential {
                service: service.to_string(),
                user: user.to_string(),
                base: self.base.clone(),
            }) as Arc<Credential>;
            Ok(Entry::new_with_credential(cred))
        }

        fn persistence(&self) -> CredentialPersistence {
            CredentialPersistence::UntilDelete
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    pub fn init(base: PathBuf) {
        let creds_dir = base.join("credentials");
        if let Ok(()) = fs::create_dir_all(&creds_dir) {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = fs::set_permissions(&creds_dir, fs::Permissions::from_mode(0o700));
            }
        }
        let store = Arc::new(FileStore { base }) as Arc<CredentialStore>;
        keyring_core::set_default_store(store);
    }
}

/// Public entry for forcing the file-backed credential store.
/// Used when native keyring is unavailable or intentionally disabled.
pub fn init_file_store(base: std::path::PathBuf) {
    tracing::info!(path = %base.display(), "using file-backed credential store");
    file_store::init(base);
}

/// Try platform-native keyring first; always fall back to encrypted-at-rest
/// file store under `app_data_dir/credentials` on any platform.
pub fn init_keyring(base: std::path::PathBuf) {
    #[cfg(target_os = "ios")]
    {
        match apple_native_keyring_store::protected::Store::new() {
            Ok(store) => {
                keyring_core::set_default_store(store);
                tracing::info!("initialized iOS keyring store");
                return;
            }
            Err(e) => tracing::warn!(error = %e, "failed to init iOS keyring store"),
        }
    }

    #[cfg(target_os = "android")]
    {
        match android_native_keyring_store::Store::new() {
            Ok(store) => {
                keyring_core::set_default_store(store);
                tracing::info!("initialized Android keyring store");
                return;
            }
            Err(e) => tracing::warn!(error = %e, "failed to init Android keyring store"),
        }
    }

    #[cfg(all(target_os = "macos", feature = "keychain"))]
    {
        match apple_native_keyring_store::keychain::Store::new() {
            Ok(store) => {
                keyring_core::set_default_store(store);
                tracing::info!("initialized macOS keychain store");
                return;
            }
            Err(e) => tracing::warn!(error = %e, "failed to init macOS keychain store"),
        }
    }

    #[cfg(target_os = "windows")]
    {
        match windows_native_keyring_store::Store::new() {
            Ok(store) => {
                keyring_core::set_default_store(store);
                tracing::info!("initialized Windows keyring store");
                return;
            }
            Err(e) => tracing::warn!(error = %e, "failed to init Windows keyring store"),
        }
    }

    #[cfg(target_os = "linux")]
    {
        match linux_keyutils_keyring_store::Store::new() {
            Ok(store) => {
                keyring_core::set_default_store(store);
                tracing::info!("initialized Linux keyutils keyring store");
                return;
            }
            Err(e) => tracing::warn!(error = %e, "failed to init Linux keyring store"),
        }
    }

    // Universal fallback for headless Linux, macOS without keychain feature,
    // failed native stores, and any future platform.
    tracing::warn!(
        path = %base.display(),
        "no native keyring available — falling back to file store"
    );
    init_file_store(base);
}