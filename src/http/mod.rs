use futures_util::StreamExt;
use sha1::{Digest as Sha1Digest, Sha1};
use sha2::{Digest as Sha2Digest, Sha256, Sha512};
use std::fs::{self, File};
use std::io::{self, BufReader, Read, Write};
use std::path::Path;

/// Enum representing supported hashers for file integrity verification.
pub enum HasherEnum {
    Sha1(Sha1),
    Sha256(Sha256),
    Sha512(Sha512),
    None,
}

impl HasherEnum {
    /// Updates the internal state of the hasher with the provided data.
    ///
    /// # Arguments
    ///
    /// * `data` - A byte slice to update the hash with.
    pub fn update(&mut self, data: &[u8]) {
        match self {
            HasherEnum::Sha1(h) => h.update(data),
            HasherEnum::Sha256(h) => h.update(data),
            HasherEnum::Sha512(h) => h.update(data),
            HasherEnum::None => {}
        }
    }

    /// Finalizes the hash computation and returns the resulting digest as a byte vector.
    pub fn finalize(self) -> Vec<u8> {
        match self {
            HasherEnum::Sha1(h) => h.finalize().to_vec(),
            HasherEnum::Sha256(h) => h.finalize().to_vec(),
            HasherEnum::Sha512(h) => h.finalize().to_vec(),
            HasherEnum::None => Vec::new(),
        }
    }
}

/// Downloads a file from the given URL and saves it to the specified path.
///
/// Optionally verifies the file's hash and can override existing files.
/// Creates parent directories as needed.
///
/// # Arguments
///
/// * `url` - The URL to download the file from.
/// * `filepath` - The local file path to save the downloaded file.
/// * `expected_hash` - Optional expected hash string for file verification.
/// * `override_file` - Whether to overwrite the file if it already exists.
///
/// # Returns
///
/// * `io::Result<()>` - Returns `Ok(())` on success, or an error if the download or verification fails.
pub async fn download_to_file(
    url: &str,
    filepath: &str,
    expected_hash: Option<&str>,
    override_file: bool,
) -> io::Result<()> {
    let expanded_path = crate::filesystem::expand_home(filepath);


    if expanded_path.exists() && !override_file {
        if let Some(expected) = expected_hash {
            if verify_hash(&expanded_path, expected)? {
                return Ok(());
            }
        } else {
            return Ok(());
        }
    }

    if let Some(parent) = expanded_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let response = reqwest::get(url)
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("http error: {}", e)))?;

    if !response.status().is_success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("download failed: status code {}", response.status()),
        ));
    }

    let mut out_file = File::create(&expanded_path)?;

    let mut hasher = match expected_hash {
        Some(h) if h.len() == 40 => HasherEnum::Sha1(Sha1::new()),
        Some(h) if h.len() == 64 => HasherEnum::Sha256(Sha256::new()),
        Some(h) if h.len() == 128 => HasherEnum::Sha512(Sha512::new()),
        _ => HasherEnum::None,
    };

    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        out_file.write_all(&chunk)?;
        hasher.update(&chunk);
    }

    if let Some(expected) = expected_hash {
        let actual = hex::encode(hasher.finalize());
        if actual != expected {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("hash mismatch: got {}, want {}", actual, expected),
            ));
        }
    }

    Ok(())
}

/// Verifies the hash of a file against an expected hash string.
///
/// Supports SHA-1, SHA-256, and SHA-512 based on the length of the expected hash.
///
/// # Arguments
///
/// * `path` - Path to the file to verify.
/// * `expected` - The expected hash string (hex-encoded).
///
/// # Returns
///
/// * `io::Result<bool>` - Returns `Ok(true)` if the hash matches, `Ok(false)` otherwise, or an error if reading fails.
pub fn verify_hash(path: &Path, expected: &str) -> io::Result<bool> {
    let f = File::open(path)?;
    let mut reader = BufReader::new(f);

    let mut hasher = match expected.len() {
        40 => HasherEnum::Sha1(Sha1::new()),
        64 => HasherEnum::Sha256(Sha256::new()),
        128 => HasherEnum::Sha512(Sha512::new()),
        _ => HasherEnum::None,
    };

    let mut buffer = [0u8; 8192];
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    let actual = hex::encode(hasher.finalize());
    Ok(actual == expected)
}