# alive_lock_file

[![crates.io](https://img.shields.io/crates/v/alive_lock_file?style=flat-square&logo=rust)](https://crates.io/crates/alive_lock_file)
[![docs.rs](https://img.shields.io/badge/docs.rs-alive_lock_file-blue?style=flat-square&logo=docs.rs)](https://docs.rs/alive_lock_file)
[![license](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](#license)

## Feature

- join the provided path to the `$XDG_RUNTIME_DIR` env variable. This directory get cleanned automatically by the system, and is mount as a ramfs.
- Atomic file creation
