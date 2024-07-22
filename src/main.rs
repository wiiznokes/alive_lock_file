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
