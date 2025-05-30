use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Represents errors that can occur during filesystem operations.
#[derive(Debug, Error)]
pub enum FilesystemError {
    /// Wrapper for standard IO errors.
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    /// Error for empty path input.
    #[error("Path is empty")]
    EmptyPath,
    /// Error when the home directory cannot be determined.
    #[error("Home directory not found")]
    HomeDirNotFound,
    /// Error for unsupported user expansion in paths (e.g., ~user).
    #[error("User expansion (~user) not supported")]
    UserExpansionNotSupported,
}

/// Options for writing files, such as whether to overwrite existing files.
pub struct WriteOptions {
    /// If true, allows overwriting an existing file.
    pub overwrite: bool,
}

impl Default for WriteOptions {
    fn default() -> Self {
        Self { overwrite: true }
    }
}

/// Options for removing files or directories, such as recursive removal.
pub struct RemoveOptions {
    /// If true, removes directories recursively.
    pub recursive: bool,
}

impl Default for RemoveOptions {
    fn default() -> Self {
        Self { recursive: false }
    }
}

/// Creates a directory if it does not exist.
///
/// # Arguments
///
/// * `dir` - Path to the directory to create.
/// * `recursive` - If true, creates parent directories as needed.
///
/// # Errors
///
/// Returns `FilesystemError` if the directory cannot be created.
pub fn create_if_not_exists<P: AsRef<Path>>(dir: P, recursive: bool) -> Result<(), FilesystemError> {
    let raw_path = dir.as_ref().to_str().ok_or(FilesystemError::EmptyPath)?;
    let path = expand_home(raw_path);

    if path.exists() {
        return Ok(());
    }

    if recursive {
        fs::create_dir_all(&path)?;
    } else {
        fs::create_dir(&path)?;
    }

    Ok(())
}

/// Checks if a directory exists at the given path.
///
/// # Arguments
///
/// * `dir` - Path to check.
///
/// # Returns
///
/// `true` if the directory exists, `false` otherwise.
pub fn dir_exists<P: AsRef<Path>>(dir: P) -> bool {
    dir.as_ref().is_dir()
}

/// Checks if a file exists at the given path.
///
/// # Arguments
///
/// * `file` - Path to check.
///
/// # Returns
///
/// `true` if the file exists, `false` otherwise.
pub fn file_exists<P: AsRef<Path>>(file: P) -> bool {
    file.as_ref().is_file()
}

/// Moves a file or directory from `src` to `dst`.
///
/// # Arguments
///
/// * `src` - Source path.
/// * `dst` - Destination path.
///
/// # Errors
///
/// Returns `FilesystemError` if the move operation fails.
pub fn move_if_exists<P: AsRef<Path>>(src: P, dst: P) -> Result<(), FilesystemError> {
    fs::rename(src, dst)?;
    Ok(())
}

/// Copies a file from `src` to `dst`, with optional overwrite.
///
/// # Arguments
///
/// * `src` - Source file path.
/// * `dst` - Destination file path.
/// * `overwrite` - If false and destination exists, returns an error.
///
/// # Errors
///
/// Returns `FilesystemError` if the copy fails or overwrite is not allowed.
///
/// # Returns
///
/// The number of bytes copied.
pub fn copy_if_exists<P: AsRef<Path>>(src: P, dst: P, overwrite: bool) -> Result<u64, FilesystemError> {
    let dst_path = dst.as_ref();
    if dst_path.exists() && !overwrite {
        return Err(FilesystemError::Io(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Destination file exists and overwrite is false",
        )));
    }
    Ok(fs::copy(src, dst_path)?)
}

/// Removes a file or directory at the given path, with options.
///
/// # Arguments
///
/// * `path` - Path to remove.
/// * `options` - Removal options (e.g., recursive).
///
/// # Errors
///
/// Returns `FilesystemError` if the removal fails.
pub fn remove_if_exists<P: AsRef<Path>>(path: P, options: RemoveOptions) -> Result<(), FilesystemError> {
    let p = path.as_ref();
    if p.is_dir() {
        if options.recursive {
            fs::remove_dir_all(p)?;
        } else {
            fs::remove_dir(p)?;
        }
    } else if p.is_file() {
        fs::remove_file(p)?;
    }
    Ok(())
}

/// Reads the contents of a file into a string.
///
/// # Arguments
///
/// * `path` - Path to the file.
///
/// # Errors
///
/// Returns `FilesystemError` if the file cannot be read.
///
/// # Returns
///
/// The file contents as a `String`.
pub fn read_file<P: AsRef<Path>>(path: P) -> Result<String, FilesystemError> {
    Ok(fs::read_to_string(path)?)
}

/// Writes content to a file, with options for overwriting.
///
/// # Arguments
///
/// * `path` - Path to the file.
/// * `content` - Content to write.
/// * `options` - Write options (e.g., overwrite).
///
/// # Errors
///
/// Returns `FilesystemError` if the write fails or overwrite is not allowed.
pub fn write_file<P: AsRef<Path>>(path: P, content: &str, options: WriteOptions) -> Result<(), FilesystemError> {
    let p = path.as_ref();
    if p.exists() && !options.overwrite {
        return Err(FilesystemError::Io(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "File exists and overwrite is false",
        )));
    }
    let mut file = fs::File::create(p)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

/// Expands a path that starts with `~` to the user's home directory.
///
/// # Arguments
///
/// * `path` - Path string, possibly starting with `~`.
///
/// # Returns
///
/// The expanded `PathBuf`, or empty if expansion fails.
pub fn expand_home(path: &str) -> PathBuf {
    if path.is_empty() {
        return PathBuf::new();
    }
    if !path.starts_with('~') {
        return PathBuf::from(path);
    }
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return PathBuf::new(),
    };
    if path == "~" {
        return home;
    }
    if path.starts_with("~/") || path.starts_with("~\\") {
        let without_tilde = &path[2..];
        return home.join(without_tilde);
    }
    PathBuf::new()
}