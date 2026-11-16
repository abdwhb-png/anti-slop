//! Read-only Git plumbing with bounded output and owned-process lifetime.
use super::{CaptureError, GIT_TIMEOUT};
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
                "core.filemode=true",
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
            let result = 'waiting: loop {
                while let Ok((is_stdout, bytes)) = receive.try_recv() {
                    match bytes {
                        Ok(bytes) if is_stdout => output = Some(bytes),
                        Ok(_) => error_output = true,
                        Err(error) => break 'waiting Err(error),
                    }
                }
                let exited = match owned.exited() {
                    Ok(exited) => exited,
                    Err(_) => {
                        break Err(CaptureError::Unavailable("Cannot observe Git exit".into()));
                    }
                };
                if exited && output.is_some() && error_output {
                    break Ok(());
                }
                if start.elapsed() >= GIT_TIMEOUT {
                    break Err(CaptureError::Timeout);
                }
                thread::sleep(Duration::from_millis(2));
            };
            // Finish all group signaling while the leader still reserves its numeric identity,
            // then reap it before joining readers (a descendant may have kept a pipe open).
            let status = owned
                .finish()
                .map_err(|_| CaptureError::Unavailable("Cannot clean up Git".into()))?;
            result?;
            if status.success() {
                Ok(output.take().expect("received stdout"))
            } else {
                Err(CaptureError::Unavailable(format!(
                    "Git {} failed; required repository data is unavailable",
                    args[0]
                )))
            }
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
    fn exited(&self) -> std::io::Result<bool> {
        let mut observation = ChildObservation {
            signal: 0,
            payload: [0; 124],
        };
        // Linux waitid(P_PID, ..., WEXITED | WNOHANG | WNOWAIT) observes exit without
        // reaping. A zombie leader keeps the PID/PGID reserved through group cleanup.
        let result = unsafe { waitid(1, self.child.id(), &mut observation, 4 | 1 | 0x01000000) };
        if result == 0 {
            return Ok(observation.signal != 0);
        }
        let error = std::io::Error::last_os_error();
        if error.kind() == std::io::ErrorKind::Interrupted {
            Ok(false)
        } else {
            Err(error)
        }
    }

    fn finish(&mut self) -> std::io::Result<std::process::ExitStatus> {
        let mut signal_error = None;
        if !self.stopped {
            self.stopped = true;
            let result = unsafe { kill(-(self.child.id() as i32), 9) };
            if result != 0 {
                let error = std::io::Error::last_os_error();
                // An exited group with no live members can report ESRCH while its leader
                // is still an unreaped zombie. Any other signaling failure stays visible.
                if error.raw_os_error() != Some(3) {
                    signal_error = Some(error);
                    let _ = self.child.kill();
                }
            }
        }
        let status = self.child.wait()?;
        if let Some(error) = signal_error {
            return Err(error);
        }
        Ok(status)
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}

// Linux siginfo_t is 128 bytes. Only its leading si_signo is read; the union stays opaque.
#[repr(C, align(8))]
struct ChildObservation {
    signal: i32,
    payload: [u8; 124],
}
unsafe extern "C" {
    fn kill(pid: i32, signal: i32) -> i32;
    fn waitid(kind: i32, id: u32, observation: *mut ChildObservation, options: i32) -> i32;
}
