//! A simple crate to create lock files without creating dead locks
//!
//! ```rs
//! use alive_lock_file::{init_signals, LockFileState};
//!
//! fn main() {
//!     // intercept the `SIGINT` and `SIGTERM` signals.
//!     init_signals();
//!
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
    mem::transmute,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use nix::sys::signal::{signal, SigHandler, Signal};

use anyhow::{anyhow, Result};

use lazy_static::lazy_static;

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

lazy_static! {
    static ref FILE_PATHS: Arc<Mutex<HashSet<PathBuf>>> = Arc::new(Mutex::new(HashSet::new()));
}

extern "C" fn handle_signal(sig: i32) {
    match FILE_PATHS.try_lock() {
        Ok(path) => {
            for path in path.iter() {
                _ = fs::remove_file(path);
            }
        }
        Err(_) => {
            // can't do much in this case
        }
    };

    let signal_number = unsafe { transmute::<i32, nix::sys::signal::Signal>(sig) };

    unsafe { signal(signal_number, SigHandler::SigDfl).unwrap() };

    unsafe { nix::libc::raise(sig) };
}

/// Intercept the `SIGINT` and `SIGTERM` signals.
/// Be warned that custom signals handler defined by other crates will be overwritten.
pub fn init_signals() {
    unsafe {
        signal(Signal::SIGINT, SigHandler::Handler(handle_signal)).unwrap();
        signal(Signal::SIGTERM, SigHandler::Handler(handle_signal)).unwrap();
    };
}

impl LockFileState {
    /// Try to acquire a lock. The name provided will be join the the runtime dir of the platform.
    /// On unix, it will be `$XDG_RUNTIME_DIR`.
    pub fn try_lock<S: AsRef<str>>(name: S) -> Result<LockFileState> {
        let path = dirs::runtime_dir()
            .ok_or(anyhow!("no runtime dir"))?
            .join(name.as_ref());

        match File::open(&path) {
            Ok(_) => return Ok(LockFileState::AlreadyLocked),
            Err(e) => {
                if e.kind() != ErrorKind::NotFound {
                    return Err(e.into());
                }
            }
        };

        let parents = path.parent().ok_or(anyhow!("no parent directory"))?;

        std::fs::create_dir_all(parents)?;
        _ = File::create(&path)?;

        FILE_PATHS.lock().unwrap().insert(path.clone());

        Ok(LockFileState::Lock(Lock { path }))
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
