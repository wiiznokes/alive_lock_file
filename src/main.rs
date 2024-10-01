use alive_lock_file::LockFileState;

fn main() {
    match LockFileState::try_lock("file.lock").unwrap() {
        LockFileState::Lock(_lock) => {
            // while _lock is in scope, `file.lock` will not be removed
        }
        LockFileState::AlreadyLocked => {}
    };
}
