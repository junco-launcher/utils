use std::fs;
use std::path::Path;
use serde::Deserialize;
use thiserror::Error;

/// Represents the contents of a `pack.mcmeta` file, which is used in Minecraft resource packs
/// to provide metadata about the pack, such as its format version and description.
#[derive(Debug, Deserialize)]
pub struct Mcmeta {
    /// The `pack` section containing format and description.
    pub pack: PackSection,
}

/// Represents the `pack` section in `pack.mcmeta`, containing the format version and description.
#[derive(Debug, Deserialize)]
pub struct PackSection {
    /// The format version of the resource pack.
    pub pack_format: u32,
    /// A description of the resource pack.
    pub description: String,
}

/// Custom error type for `parse_resource_pack_mcmeta`.
#[derive(Debug, Error)]
pub enum McmetaError {
    #[error("Failed to read the file: {0}")]
    FileReadError(#[from] std::io::Error),
    #[error("Failed to parse JSON: {0}")]
    JsonParseError(#[from] serde_json::Error),
    #[error("Missing `pack` section in the mcmeta file")]
    MissingPackSection,
}

/// Parses a `pack.mcmeta` file and returns its contents as an `Mcmeta` struct.
///
/// # Arguments
///
/// * `path` - A path to the `pack.mcmeta` file to parse.
///
/// # Returns
///
/// * `Ok(Mcmeta)` if the file is successfully read and parsed.
/// * `Err(McmetaError)` if there is an error reading the file or parsing its contents.
///
/// # Errors
///
/// Returns an error if the file cannot be read, if the contents cannot be deserialized as JSON,
/// or if the `pack` section is missing.
pub fn parse_resource_pack_mcmeta<P: AsRef<Path>>(path: P) -> Result<Mcmeta, McmetaError> {
    let content = fs::read_to_string(path)?;
    let mcmeta: Mcmeta = serde_json::from_str(&content)?;

    // Validate that the `pack` section exists
    if mcmeta.pack.pack_format == 0 || mcmeta.pack.description.is_empty() {
        return Err(McmetaError::MissingPackSection);
    }

    Ok(mcmeta)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    fn write_temp_mcmeta(content: &str) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("pack.mcmeta");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        (dir, file_path)
    }

    #[test]
    fn parses_valid_mcmeta_file() {
        let mcmeta_content = r#"{
        "pack": {
            "pack_format": 6,
            "description": "A test resource pack"
        }
    }"#;
        let (_dir, file_path) = write_temp_mcmeta(mcmeta_content);
        let result = parse_resource_pack_mcmeta(&file_path);
        if let Err(e) = &result {
            println!("error parsing mcmeta: {:?}", e);
        }
        assert!(result.is_ok());
        let mcmeta = result.unwrap();
        assert_eq!(mcmeta.pack.pack_format, 6);
        assert_eq!(mcmeta.pack.description, "A test resource pack");
    }

    #[test]
    fn returns_error_for_missing_file() {
        let result = parse_resource_pack_mcmeta("non_existent_file.mcmeta");
        assert!(result.is_err());
    }

    #[test]
    fn returns_error_for_invalid_json() {
        let mcmeta_content = r#"{
            "pack": {
                "pack_format": "not_a_number",
                "description": "Invalid format"
            }
        }"#;
        let (_dir, file_path) = write_temp_mcmeta(mcmeta_content);
        let result = parse_resource_pack_mcmeta(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn returns_error_for_missing_pack_section() {
        let mcmeta_content = r#"{
            "not_pack": {
                "pack_format": 6,
                "description": "Missing pack section"
            }
        }"#;
        let (_dir, file_path) = write_temp_mcmeta(mcmeta_content);
        let result = parse_resource_pack_mcmeta(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn returns_error_for_empty_file() {
        let (_dir, file_path) = write_temp_mcmeta("");
        let result = parse_resource_pack_mcmeta(&file_path);
        assert!(result.is_err());
    }
}