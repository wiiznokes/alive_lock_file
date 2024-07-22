//! A simple crate to create lock file without creating dead locks
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
    path::PathBuf,
    sync::{Arc, Mutex},
};

use nix::sys::signal::{signal, SigHandler, Signal};

use thiserror::Error;

use anyhow::anyhow;

use lazy_static::lazy_static;

#[derive(Error, Debug)]
pub enum LockFileError {
    #[error(transparent)]
    Error(#[from] anyhow::Error),
}

#[derive(Debug, Clone)]
pub enum LockFileState {
    Lock(Lock),
    AlreadyLocked,
}

#[derive(Debug, Clone)]
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
    pub fn try_lock<S: AsRef<str>>(name: S) -> Result<LockFileState, LockFileError> {
        let path = dirs::runtime_dir()
            .ok_or(anyhow!("no runtime dir"))?
            .join(name.as_ref());

        match File::open(&path) {
            Ok(_) => return Ok(LockFileState::AlreadyLocked),
            Err(e) => {
                if e.kind() != ErrorKind::NotFound {
                    return Err(LockFileError::Error(e.into()));
                }
            }
        };

        _ = File::create(&path).map_err(|e| LockFileError::Error(e.into()))?;

        FILE_PATHS.lock().unwrap().insert(path.clone());

        Ok(LockFileState::Lock(Lock { path }))
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        _ = fs::remove_file(&self.path);

        FILE_PATHS.lock().unwrap().remove(&self.path);
    }
}
