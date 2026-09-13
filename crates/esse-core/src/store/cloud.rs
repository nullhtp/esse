//! Files that live in the cloud rather than on the disk.
//!
//! `~/Documents` is one of the folders iCloud Drive syncs, and when the disk
//! gets tight macOS evicts files out of it: the name and the size stay, the
//! bytes go, and the file is flagged `dataless`. Opening one fetches it back —
//! but only in a process that is allowed to wait for the download. A process
//! that is not allowed gets `EDEADLK`, "Resource deadlock avoided", the moment
//! it opens the file, and never learns that the reason is a download it was
//! not permitted to make.
//!
//! Permission is inherited, and launchd does not hand it out: esse started
//! from a terminal reads an evicted `sparks.jsonl` without noticing, while the
//! same binary started at login by the launch agent cannot open it at all.
//! That is the whole of the bug this module closes. esse's folder is the
//! writer's folder, and every file in it is worth waiting for.

#[cfg(target_os = "macos")]
use std::os::raw::c_int;
#[cfg(target_os = "macos")]
use std::sync::Once;

// <sys/resource.h>: which policy, whose process it applies to, and the value
// that means "wait for the download".
#[cfg(target_os = "macos")]
const IOPOL_TYPE_VFS_MATERIALIZE_DATALESS_FILES: c_int = 3;
#[cfg(target_os = "macos")]
const IOPOL_SCOPE_PROCESS: c_int = 0;
#[cfg(target_os = "macos")]
const IOPOL_MATERIALIZE_DATALESS_FILES_ON: c_int = 2;

#[cfg(target_os = "macos")]
extern "C" {
    fn setiopolicy_np(iotype: c_int, scope: c_int, policy: c_int) -> c_int;
}

/// Lets this process wait for evicted files to come back down from the cloud.
///
/// Said once, for the whole process, before the first file is opened. Nothing
/// is reported if the system refuses: the only consequence is the error the
/// read would have raised anyway, and that one names the file.
pub(crate) fn allow_downloads() {
    #[cfg(target_os = "macos")]
    {
        static ONCE: Once = Once::new();
        ONCE.call_once(|| {
            // Safety: three integers into a libc call that only reads them.
            unsafe {
                setiopolicy_np(
                    IOPOL_TYPE_VFS_MATERIALIZE_DATALESS_FILES,
                    IOPOL_SCOPE_PROCESS,
                    IOPOL_MATERIALIZE_DATALESS_FILES_ON,
                );
            }
        });
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    extern "C" {
        fn getiopolicy_np(iotype: c_int, scope: c_int) -> c_int;
    }

    /// The launch-agent bug in the one form a test can see it. A real evicted
    /// file needs an iCloud account and a full disk; what the test can check is
    /// that the process has come down on the side of waiting for the bytes,
    /// which is the whole difference between reading the folder and `EDEADLK`.
    #[test]
    fn the_process_waits_for_evicted_files() {
        allow_downloads();

        // Safety: two integers into a libc call that only reads them.
        let policy = unsafe {
            getiopolicy_np(
                IOPOL_TYPE_VFS_MATERIALIZE_DATALESS_FILES,
                IOPOL_SCOPE_PROCESS,
            )
        };

        assert_eq!(
            policy, IOPOL_MATERIALIZE_DATALESS_FILES_ON,
            "an evicted file would fail to open with EDEADLK"
        );
    }
}
