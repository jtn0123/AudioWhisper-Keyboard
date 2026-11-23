//! Where the stores' files live, and how a file is replaced so that a crash at any moment leaves
//! either the old file or the new one.
//!
//! [`save`] writes the new bytes to `<path>.tmp`, syncs them, renames the temporary file over
//! `path` and syncs the directory, so the rename itself is on disk before the change counts as
//! made. [`load`] first removes a `<path>.tmp` a crash left behind: it was never renamed, so the
//! file at `path` is still the last whole one. The steps go through [`Disk`], which the tests
//! replace with one that can stop the process at any step.

use std::fs::{self, File, OpenOptions};
use std::io::{self, ErrorKind, Write};
use std::path::{Path, PathBuf};

/// The file operations a store makes, each one step a crash can come between.
pub trait Disk: Send {
    /// The whole file at `path`, or `None` if there is none.
    fn read(&mut self, path: &Path) -> io::Result<Option<Vec<u8>>>;
    /// Creates or truncates the file at `path`, writes `bytes` and syncs them.
    fn write_synced(&mut self, path: &Path, bytes: &[u8]) -> io::Result<()>;
    /// Renames `from` over `to`.
    fn rename(&mut self, from: &Path, to: &Path) -> io::Result<()>;
    /// Syncs the directory `path`, so renames and removals in it are on disk.
    fn sync_dir(&mut self, path: &Path) -> io::Result<()>;
    /// Removes the file at `path`; none there is no error.
    fn remove(&mut self, path: &Path) -> io::Result<()>;
}

/// The real file system.
#[derive(Debug, Default)]
pub struct RealDisk;

impl Disk for RealDisk {
    fn read(&mut self, path: &Path) -> io::Result<Option<Vec<u8>>> {
        match fs::read(path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn write_synced(&mut self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        file.write_all(bytes)?;
        file.sync_all()
    }

    fn rename(&mut self, from: &Path, to: &Path) -> io::Result<()> {
        fs::rename(from, to)
    }

    fn sync_dir(&mut self, path: &Path) -> io::Result<()> {
        File::open(path)?.sync_all()
    }

    fn remove(&mut self, path: &Path) -> io::Result<()> {
        match fs::remove_file(path) {
            Err(e) if e.kind() != ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        }
    }
}

/// The temporary file [`save`] writes before renaming it to `path`.
pub fn temporary(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(".tmp");
    PathBuf::from(name)
}

/// The directory holding `path`, which [`save`] syncs.
fn directory(path: &Path) -> &Path {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent,
        _ => Path::new("."),
    }
}

/// Replaces the file at `path` with `bytes`: temporary file, sync, rename, directory sync.
pub fn save(disk: &mut dyn Disk, path: &Path, bytes: &[u8]) -> io::Result<()> {
    let temporary = temporary(path);
    disk.write_synced(&temporary, bytes)?;
    disk.rename(&temporary, path)?;
    disk.sync_dir(directory(path))
}

/// The file at `path`, or `None` if there is none, once a temporary file a crash left is gone.
pub fn load(disk: &mut dyn Disk, path: &Path) -> io::Result<Option<Vec<u8>>> {
    disk.remove(&temporary(path))?;
    disk.read(path)
}

/// Removes the files at `paths` in order, syncing the directory after each, so that a
/// database's main file is gone for good before its journal goes.
pub fn remove_all(disk: &mut dyn Disk, paths: &[PathBuf]) -> io::Result<()> {
    for path in paths {
        disk.remove(path)?;
        disk.sync_dir(directory(path))?;
    }
    Ok(())
}
