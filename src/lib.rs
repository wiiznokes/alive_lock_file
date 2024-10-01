//! A simple crate to create lock files without creating dead locks
//!
//! ```rs
//! use alive_lock_file::LockFileState;
//!
//! fn main() {
//!     match LockFileState::try_lock("file.lock").unwrap() {
//!         LockFileState::Lock(_lock) => {
//!             // while _lock is in scope, `file.lock` will not be removed
//!         }
//!         LockFileState::AlreadyLocked => {}
//!     };
//! }
//! ```

use std::{
    collections::HashSet,
    fs::{self, File},
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::{LazyLock, Mutex},
};

use anyhow::{anyhow, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockFileState {
    Lock(Lock),
    AlreadyLocked,
}

/// Represent a lock file. When this value is dropped, the corresponding lock file
/// will be removed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lock {
    path: PathBuf,
}

static FILE_PATHS: LazyLock<Mutex<HashSet<PathBuf>>> = LazyLock::new(|| Mutex::new(HashSet::new()));

impl LockFileState {
    /// Try to acquire a lock. The name provided will be join the the runtime dir of the platform.
    /// On unix, it will be `$XDG_RUNTIME_DIR`.
    pub fn try_lock<S: AsRef<str>>(name: S) -> Result<LockFileState> {
        let path = dirs::runtime_dir()
            .ok_or(anyhow!("no runtime dir"))?
            .join(name.as_ref());

        let parents = path.parent().ok_or(anyhow!("no parent directory"))?;

        std::fs::create_dir_all(parents)?;

        match File::create_new(&path) {
            Ok(_) => {}
            Err(e) => {
                if e.kind() == ErrorKind::AlreadyExists {
                    return Ok(LockFileState::AlreadyLocked);
                }
                return Err(e.into());
            }
        }

        FILE_PATHS.lock().unwrap().insert(path.clone());

        Ok(LockFileState::Lock(Lock { path }))
    }

    pub fn has_lock(&self) -> bool {
        match self {
            LockFileState::Lock(_) => true,
            LockFileState::AlreadyLocked => false,
        }
    }
}

impl Lock {
    /// Get the path of this lock file.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        _ = fs::remove_file(&self.path);

        FILE_PATHS.lock().unwrap().remove(&self.path);
    }
}
