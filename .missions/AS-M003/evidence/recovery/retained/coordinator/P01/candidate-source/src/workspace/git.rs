//! Read-only Git plumbing with bounded output and owned-process lifetime.
use super::{CaptureError, GIT_TIMEOUT, MAX_INVENTORY_BYTES};
use std::os::unix::process::CommandExt;
use std::{
    io::Read,
    path::Path,
    process::{Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

pub(super) struct Git {
    pub processes: usize,
}

impl Git {
    pub fn read(
        &mut self,
        root: &Path,
        args: &[&str],
        limit: usize,
    ) -> Result<Vec<u8>, CaptureError> {
        self.processes += 1;
        let mut command = Command::new("git");
        command
            .current_dir(root)
            .env_clear()
            .env("PATH", std::env::var_os("PATH").unwrap_or_default())
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_NO_LAZY_FETCH", "1")
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_OPTIONAL_LOCKS", "0")
            .env("GIT_ATTR_NOSYSTEM", "1")
            .args([
                "--no-replace-objects",
                "-c",
                "core.fsmonitor=false",
                "-c",
                "core.hooksPath=/dev/null",
                "-c",
                "diff.external=",
            ])
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0);
        let child = command
            .spawn()
            .map_err(|_| CaptureError::Unavailable("Cannot start Git".into()))?;
        let mut owned = OwnedChild {
            child,
            stopped: false,
        };
        let stdout = owned.child.stdout.take().expect("piped stdout");
        let stderr = owned.child.stderr.take().expect("piped stderr");
        thread::scope(|scope| {
            let (send, receive) = mpsc::channel();
            let other = send.clone();
            scope.spawn(move || {
                let _ = send.send((true, bounded(stdout, limit)));
            });
            scope.spawn(move || {
                let _ = other.send((false, bounded(stderr, 8192)));
            });
            let start = Instant::now();
            let mut output = None;
            let mut error_output = false;
            let mut status = None;
            let result = loop {
                while let Ok((is_stdout, bytes)) = receive.try_recv() {
                    match bytes {
                        Ok(bytes) if is_stdout => output = Some(bytes),
                        Ok(_) => error_output = true,
                        Err(error) => {
                            owned.stop();
                            return Err(error);
                        }
                    }
                }
                if status.is_none() {
                    status = match owned.child.try_wait() {
                        Ok(status) => status,
                        Err(_) => {
                            owned.stop();
                            return Err(CaptureError::Unavailable("Cannot wait for Git".into()));
                        }
                    };
                }
                if let Some(status) = status
                    && output.is_some()
                    && error_output
                {
                    break if status.success() {
                        Ok(output.take().expect("received stdout"))
                    } else {
                        Err(CaptureError::Unavailable(format!(
                            "Git {} failed; required repository data is unavailable",
                            args[0]
                        )))
                    };
                }
                if start.elapsed() >= GIT_TIMEOUT {
                    break Err(CaptureError::Timeout);
                }
                thread::sleep(Duration::from_millis(2));
            };
            // Kill the whole owned group before scoped readers are joined, including on timeout.
            owned.stop();
            result
        })
    }

    pub fn blob(&mut self, root: &Path, oid: &str, limit: usize) -> Result<Vec<u8>, CaptureError> {
        let header = self.read(root, &["cat-file", "-s", oid], 32)?;
        let size = std::str::from_utf8(&header)
            .ok()
            .and_then(|s| s.trim().parse::<usize>().ok())
            .ok_or(CaptureError::Unsupported("Invalid Git object size"))?;
        if size > limit {
            return Err(CaptureError::Limit("Git blob bytes"));
        }
        let bytes = self.read(root, &["cat-file", "blob", oid], size)?;
        if bytes.len() != size {
            return Err(CaptureError::Unavailable("Git blob size changed".into()));
        }
        Ok(bytes)
    }

    pub fn inventory(&mut self, root: &Path, head: &str) -> Result<Vec<u8>, CaptureError> {
        self.read(
            root,
            &[
                "diff-index",
                "--cached",
                "--raw",
                "--no-renames",
                "--no-relative",
                "--no-ext-diff",
                "--no-textconv",
                "--ignore-submodules=none",
                "--abbrev=64",
                "-z",
                head,
                "--",
            ],
            MAX_INVENTORY_BYTES,
        )
    }
}

fn bounded(mut input: impl Read, limit: usize) -> Result<Vec<u8>, CaptureError> {
    let mut bytes = Vec::new();
    let mut chunk = [0; 8192];
    loop {
        let available = chunk.len().min(limit.saturating_sub(bytes.len()) + 1);
        let count = input
            .read(&mut chunk[..available])
            .map_err(|_| CaptureError::Unavailable("Cannot read Git output".into()))?;
        if count == 0 {
            return Ok(bytes);
        }
        if count > limit - bytes.len() {
            return Err(CaptureError::Limit("Git output bytes"));
        }
        bytes.extend_from_slice(&chunk[..count]);
    }
}

struct OwnedChild {
    child: std::process::Child,
    stopped: bool,
}
impl OwnedChild {
    fn stop(&mut self) {
        if self.stopped {
            return;
        }
        self.stopped = true;
        // Each child starts a new process group. Negative PID addresses only that owned group.
        unsafe {
            kill(-(self.child.id() as i32), 9);
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        self.stop();
    }
}
unsafe extern "C" {
    fn kill(pid: i32, signal: i32) -> i32;
}
