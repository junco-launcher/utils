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
    #[inline]
    pub fn update(&mut self, data: &[u8]) {
        match self {
            HasherEnum::Sha1(h) => h.update(data),
            HasherEnum::Sha256(h) => h.update(data),
            HasherEnum::Sha512(h) => h.update(data),
            HasherEnum::None => {}
        }
    }

    /// Finalizes the hash computation and returns the resulting digest as a byte vector.
    #[inline]
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write as IoWrite;
    use tempfile::tempdir;

    #[tokio::test]
    async fn download_to_file_saves_file_and_verifies_hash() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("file.txt");
        let content = b"hello world";
        let hash = hex::encode(sha1::Sha1::digest(content));

        let server = httpmock::MockServer::start();
        let mock = server.mock(|when, then| {
            when.method("GET").path("/file.txt");
            then.status(200)
                .header("content-type", "application/octet-stream")
                .body(content);
        });

        download_to_file(
            &format!("{}/file.txt", server.url("")),
            file_path.to_str().unwrap(),
            Some(&hash),
            true,
        )
            .await
            .unwrap();

        let file_content = fs::read(&file_path).unwrap();
        assert_eq!(file_content, content);
        mock.assert();
    }

    #[tokio::test]
    async fn download_to_file_returns_error_on_hash_mismatch() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("file.txt");
        let content = b"hello world";
        let wrong_hash = "0000000000000000000000000000000000000000";

        let server = httpmock::MockServer::start();
        server.mock(|when, then| {
            when.method("GET").path("/file.txt");
            then.status(200).body(content);
        });

        let result = download_to_file(
            &format!("{}/file.txt", server.url("")),
            file_path.to_str().unwrap(),
            Some(wrong_hash),
            true,
        )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn download_to_file_skips_download_if_file_exists_and_hash_matches() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("file.txt");
        let content = b"hello world";
        let hash = hex::encode(sha1::Sha1::digest(content));
        let mut f = File::create(&file_path).unwrap();
        f.write_all(content).unwrap();

        let server = httpmock::MockServer::start();
        let mock = server.mock(|when, then| {
            when.method("GET").path("/file.txt");
            then.status(200).body("should not be called");
        });

        download_to_file(
            &format!("{}/file.txt", server.url("")),
            file_path.to_str().unwrap(),
            Some(&hash),
            false,
        )
            .await
            .unwrap();

        mock.assert_hits(0);
    }

    #[tokio::test]
    async fn download_to_file_creates_parent_directories() {
        let dir = tempdir().unwrap();
        let nested_path = dir.path().join("a/b/c/file.txt");
        let content = b"abc";
        let server = httpmock::MockServer::start();
        server.mock(|when, then| {
            when.method("GET").path("/file.txt");
            then.status(200).body(content);
        });

        download_to_file(
            &format!("{}/file.txt", server.url("")),
            nested_path.to_str().unwrap(),
            None,
            true,
        )
            .await
            .unwrap();

        assert!(nested_path.exists());
        let file_content = fs::read(&nested_path).unwrap();
        assert_eq!(file_content, content);
    }

    #[tokio::test]
    async fn download_to_file_returns_error_on_http_failure() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("file.txt");
        let server = httpmock::MockServer::start();
        server.mock(|when, then| {
            when.method("GET").path("/file.txt");
            then.status(404);
        });

        let result = download_to_file(
            &format!("{}/file.txt", server.url("")),
            file_path.to_str().unwrap(),
            None,
            true,
        )
            .await;

        assert!(result.is_err());
    }

    #[test]
    fn verify_hash_returns_true_on_matching_hash() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("file.txt");
        let content = b"hash me";
        let mut f = File::create(&file_path).unwrap();
        f.write_all(content).unwrap();
        let hash = hex::encode(sha1::Sha1::digest(content));
        assert!(verify_hash(&file_path, &hash).unwrap());
    }

    #[test]
    fn verify_hash_returns_false_on_non_matching_hash() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("file.txt");
        let content = b"hash me";
        let mut f = File::create(&file_path).unwrap();
        f.write_all(content).unwrap();
        let wrong_hash = "0000000000000000000000000000000000000000";
        assert!(!verify_hash(&file_path, wrong_hash).unwrap());
    }

    #[test]
    fn verify_hash_returns_true_for_sha256_and_sha512() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("file.txt");
        let content = b"abc123";
        let mut f = File::create(&file_path).unwrap();
        f.write_all(content).unwrap();

        let sha256 = hex::encode(sha2::Sha256::digest(content));
        let sha512 = hex::encode(sha2::Sha512::digest(content));
        assert!(verify_hash(&file_path, &sha256).unwrap());
        assert!(verify_hash(&file_path, &sha512).unwrap());
    }

    #[test]
    fn verify_hash_returns_true_when_expected_is_empty_and_file_is_empty() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("file.txt");
        File::create(&file_path).unwrap();
        assert!(verify_hash(&file_path, "").unwrap());
    }
}