use std::{
    fs::{self, File},
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use log::error;

#[must_use]
pub enum LockResult {
    Success,
    AlreadyLocked,
}

#[must_use]
pub enum LockResultWithDrop {
    Locked(Lock),
    AlreadyLocked,
}

impl LockResultWithDrop {
    pub fn has_lock(&self) -> bool {
        matches!(self, Self::Locked(_))
    }
}

/// Represent a lock file. When this value is dropped, the corresponding lock file will be removed.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct Lock {
    path: PathBuf,
}

/// Remove the lock if exist. Return true if successfully removed, false if there was no lock.
pub fn remove_lock<S: AsRef<str>>(name: S) -> Result<bool> {
    let path = get_lock_path(name.as_ref())?;

    if fs::exists(&path)? {
        fs::remove_file(&path)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Try to acquire the lock.
pub fn try_lock<S: AsRef<str>>(name: S) -> Result<LockResult> {
    let path = get_lock_path(name.as_ref())?;
    let res = create_log_file(&path)?;
    Ok(res)
}

/// Return true if this name is locked.
pub fn is_locked<S: AsRef<str>>(name: S) -> Result<bool> {
    let path = get_lock_path(name.as_ref())?;
    let exist = fs::exists(&path)?;
    Ok(exist)
}

/// Try to acquire the lock, and unlock when the [`Lock`] is dropped.
pub fn try_lock_until_dropped<S: AsRef<str>>(name: S) -> Result<LockResultWithDrop> {
    let path = get_lock_path(name.as_ref())?;
    let res = create_log_file(&path)?;
    let res = match res {
        LockResult::Success => LockResultWithDrop::Locked(Lock { path }),
        LockResult::AlreadyLocked => LockResultWithDrop::AlreadyLocked,
    };
    Ok(res)
}

impl Lock {
    /// Get the path of this lock file.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        if let Err(e) = fs::remove_file(&self.path) {
            error!(
                "can't remove file {} in drop lock: {e}",
                self.path.display()
            );
        }
    }
}

fn get_lock_path(name: &str) -> Result<PathBuf> {
    let path = dirs::runtime_dir()
        .ok_or(anyhow!("no runtime dir"))?
        .join(name);

    Ok(path)
}

fn create_log_file(path: &Path) -> Result<LockResult> {
    let parents = path.parent().ok_or(anyhow!("no parent directory"))?;

    std::fs::create_dir_all(parents)?;

    match File::create_new(&path) {
        Ok(_) => Ok(LockResult::Success),
        Err(e) => {
            if e.kind() == ErrorKind::AlreadyExists {
                return Ok(LockResult::AlreadyLocked);
            }
            return Err(e.into());
        }
    }
}
