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

use nix::{libc, sys::signal::{signal, SigHandler, Signal}};

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

extern fn handle_signal(sig: libc::c_int) {
    match FILE_PATHS.try_lock() {
        Ok(path) => {
            log::debug!("{:?}", path);

            for path in path.iter() {
                if let Err(err) = fs::remove_file(path) {
                    log::error!("can't remove lock file {}", err);
                }
            }
        }
        Err(_) => {
            log::error!("can't get the lock");
            
        }
    };

    let signal_number = unsafe { transmute::<i32, nix::sys::signal::Signal>(sig) };

    log::debug!("handle_signal: {:?}", signal_number);

    unsafe { signal(signal_number, SigHandler::SigDfl).unwrap() };

    unsafe { nix::libc::raise(sig) };
}

/// Intercept the `SIGINT` and `SIGTERM` signals.
/// Be warned that custom signals handler defined by other crates will be overwritten.
pub fn init_signals() {
    // chat gpt generated
    let fatal_signals = [
        Signal::SIGHUP,
        Signal::SIGINT,
        Signal::SIGQUIT,
        Signal::SIGILL,
        Signal::SIGABRT,
        Signal::SIGBUS,
        Signal::SIGFPE,
        Signal::SIGKILL,
        Signal::SIGSEGV,
        Signal::SIGPIPE,
        Signal::SIGALRM,
        Signal::SIGTERM,
        Signal::SIGXCPU,
        Signal::SIGXFSZ,
        Signal::SIGSYS,
    ];

    for sig in fatal_signals {
        unsafe {
            signal(sig, SigHandler::Handler(handle_signal)).unwrap();
        }
    }
}

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
