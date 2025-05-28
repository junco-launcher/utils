/// The `options_parser` module provides functionality for parsing configuration
/// or options data from text input. It includes types and functions to interpret
/// various data types (such as integers, floats, booleans, strings, and lists)
/// from lines of text.
///
/// Typical usage involves calling the parsing functions to convert lines or files
/// into structured data for further processing.
pub mod options_parser;

/// The `mcmeta_parser` module is responsible for parsing `.mcmeta` files,
/// which are commonly used in Minecraft resource packs and data packs to
/// describe metadata such as pack format and description.
///
/// This module provides types and functions to read and interpret `.mcmeta` files
/// into Rust data structures.
pub mod mcmeta_parser;

/// Filesystem utilities for common tasks in a clean and cross-platform way.
///
/// # Features
/// - create/remove files or directories (optionally recursive)
/// - check if files or directories exist
/// - read/write files with options
/// - move/copy files with optional overwrite
/// - expand `~` to home directory
/// - custom error type for better error handling
///
/// # Errors
/// Everything returns a `Result<T, FilesystemError>`.
///
/// # Compatibility
/// Uses `Path`, `PathBuf`, and `dirs` for home dir expansion. Works on all platforms.
///
/// # Example
/// ```rust
/// use crate::junco_launcher_utils::filesystem::{create_if_not_exists, write_file, WriteOptions};
///
/// create_if_not_exists("my_dir", true)?;
/// write_file("my_dir/hello.txt", "hello", WriteOptions::default())?;
/// ```
pub mod filesystem;

pub mod http;