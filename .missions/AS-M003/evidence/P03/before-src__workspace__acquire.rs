//! Linux descriptor-anchored source reads. No source symlink is followed, even during a swap.
use super::{CaptureError, GitMode, MAX_FILE_BYTES, MAX_FILES, MAX_TOTAL_BYTES};
use std::{
    fs::{File, OpenOptions},
    io::Read,
    os::fd::AsRawFd,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    sync::Arc,
};

// Linux asm-generic/fcntl.h. NONBLOCK avoids hanging on a regular-file-to-FIFO swap.
const NOFOLLOW: i32 = 0o400000;
const DIRECTORY: i32 = 0o200000;
const NONBLOCK: i32 = 0o4000;

#[derive(Clone, PartialEq, Eq)]
pub(super) struct WorkingFile {
    pub bytes: Arc<[u8]>,
    pub mode: GitMode,
}

pub(super) struct Acquisition {
    root: File,
    pub files: usize,
    pub bytes: usize,
}
impl Acquisition {
    pub fn new(root: &Path) -> Result<Self, CaptureError> {
        let root = OpenOptions::new()
            .read(true)
            .custom_flags(NOFOLLOW | DIRECTORY)
            .open(root)
            .map_err(|_| CaptureError::Unavailable("Cannot open workspace directory".into()))?;
        Ok(Self {
            root,
            files: 0,
            bytes: 0,
        })
    }
    pub fn cwd(&self) -> PathBuf {
        PathBuf::from(format!("/proc/self/fd/{}", self.root.as_raw_fd()))
    }
    pub fn remaining(&self) -> usize {
        MAX_FILE_BYTES.min(MAX_TOTAL_BYTES - self.bytes)
    }
    pub fn admit_file(&mut self) -> Result<(), CaptureError> {
        if self.files >= MAX_FILES {
            return Err(CaptureError::Limit("Retained input count"));
        }
        self.files += 1;
        Ok(())
    }
    pub fn admit_bytes(&mut self, count: usize) -> Result<(), CaptureError> {
        if count > self.remaining() {
            return Err(CaptureError::Limit("Retained input bytes"));
        }
        self.bytes += count;
        Ok(())
    }
    pub fn working(&self, path: &str, limit: usize) -> Result<Option<WorkingFile>, CaptureError> {
        let mut directory = self
            .root
            .try_clone()
            .map_err(|_| CaptureError::Unavailable(path.into()))?;
        let mut components = Path::new(path).components().peekable();
        while let Some(component) = components.next() {
            let next = PathBuf::from(format!("/proc/self/fd/{}", directory.as_raw_fd()))
                .join(component.as_os_str());
            let flags = NOFOLLOW
                | NONBLOCK
                | if components.peek().is_some() {
                    DIRECTORY
                } else {
                    0
                };
            directory = match OpenOptions::new().read(true).custom_flags(flags).open(next) {
                Ok(file) => file,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(_) => {
                    return Err(CaptureError::Unavailable(format!(
                        "Required path is unavailable or contains a symlink: {path}"
                    )));
                }
            };
        }
        let metadata = directory
            .metadata()
            .map_err(|_| CaptureError::Unavailable(path.into()))?;
        let mode = if metadata.permissions().mode() & 0o111 == 0 {
            GitMode::Regular
        } else {
            GitMode::Executable
        };
        Ok(Some(WorkingFile {
            bytes: read_file(directory, limit)?.into(),
            mode,
        }))
    }

    pub fn configuration(&self, file: &Path, limit: usize) -> Result<Vec<u8>, CaptureError> {
        let input = OpenOptions::new()
            .read(true)
            .custom_flags(NOFOLLOW | NONBLOCK)
            .open(file)
            .map_err(|_| CaptureError::Unavailable("Cannot read selected configuration".into()))?;
        read_file(input, limit)
    }
}

fn read_file(mut input: File, limit: usize) -> Result<Vec<u8>, CaptureError> {
    let metadata = input
        .metadata()
        .map_err(|_| CaptureError::Unavailable("Cannot inspect required input".into()))?;
    if !metadata.is_file() {
        return Err(CaptureError::Unsupported(
            "Required input must be a regular file",
        ));
    }
    if metadata.len() > limit as u64 {
        return Err(CaptureError::Limit("Required input bytes"));
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    let mut chunk = [0; 8192];
    loop {
        let available = chunk.len().min(limit.saturating_sub(bytes.len()) + 1);
        let count = input
            .read(&mut chunk[..available])
            .map_err(|_| CaptureError::Unavailable("Cannot read required input".into()))?;
        if count == 0 {
            return Ok(bytes);
        }
        if count > limit - bytes.len() {
            return Err(CaptureError::Limit("Required input grew beyond byte limit"));
        }
        bytes.extend_from_slice(&chunk[..count]);
    }
}
