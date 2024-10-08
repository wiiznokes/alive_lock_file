# alive_lock_file

[![crates.io](https://img.shields.io/crates/v/alive_lock_file?style=flat-square&logo=rust)](https://crates.io/crates/alive_lock_file)
[![docs.rs](https://img.shields.io/badge/docs.rs-alive_lock_file-blue?style=flat-square&logo=docs.rs)](https://docs.rs/alive_lock_file)
[![license](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](#license)

A simple crate to create lock files without creating dead locks. For that, it use 2 things:
- join the provided path to the `$XDG_RUNTIME_DIR` env variable. This directory get cleanned automatically by the system, and is mount as a ramfs.
- intercept signals

## Usage

```rs
use alive_lock_file::{init_signals, LockFileState};

fn main() {
    // intercept the `SIGINT` and `SIGTERM` signals.
    init_signals();

    match LockFileState::try_lock("file.lock").unwrap() {
        LockFileState::Lock(_lock) => {
            // while _lock is in scope, `file.lock` will not be removed
        }
        LockFileState::AlreadyLocked => {}
    };
}
```
