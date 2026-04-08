use flate2::read::GzDecoder;
use std::path::{Path, PathBuf};
use std::process::Command;
use tar::Archive;
use tempfile::TempDir;

/// Extracts a .tar.gz file to a temp directory and returns the path to the .pgsql file found inside.
pub fn extract_pgsql(tar_gz_path: &str) -> Result<(TempDir, PathBuf), String> {
    let path = Path::new(tar_gz_path);
    if !path.exists() {
        return Err(format!("File not found: {}", tar_gz_path));
    }

    let file =
        std::fs::File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);

    let tmp_dir =
        TempDir::new().map_err(|e| format!("Failed to create temp directory: {}", e))?;

    archive
        .unpack(tmp_dir.path())
        .map_err(|e| format!("Failed to extract tar.gz: {}", e))?;

    // Find the .pgsql file at the root of the archive
    let pgsql_file = find_pgsql_file(tmp_dir.path())?;

    Ok((tmp_dir, pgsql_file))
}

/// Finds a .pgsql file in the given directory (non-recursive, root level only).
fn find_pgsql_file(dir: &Path) -> Result<PathBuf, String> {
    let entries =
        std::fs::read_dir(dir).map_err(|e| format!("Failed to read temp directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "pgsql" {
                    return Ok(path);
                }
            }
        }
    }

    Err("No .pgsql file found in the tar.gz archive".to_string())
}

/// Runs pg_restore against the given database using the connection string.
pub fn run_pg_restore(pgsql_path: &Path, connection_string: &str) -> Result<String, String> {
    let output = Command::new("pg_restore")
        .arg("--no-owner")
        .arg("--no-privileges")
        .arg("-d")
        .arg(connection_string)
        .arg(pgsql_path)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                "pg_restore not found. Please install PostgreSQL client tools.".to_string()
            } else {
                format!("Failed to run pg_restore: {}", e)
            }
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        let mut result = "Restore completed successfully.".to_string();
        if !stderr.is_empty() {
            result.push_str(&format!("\nWarnings:\n{}", stderr));
        }
        Ok(result)
    } else {
        Err(format!(
            "pg_restore failed (exit code: {}):\n{}{}",
            output
                .status
                .code()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            stderr,
            if !stdout.is_empty() {
                format!("\nstdout:\n{}", stdout)
            } else {
                String::new()
            }
        ))
    }
}

/// Full restore pipeline: extract .tar.gz → find .pgsql → run pg_restore.
pub fn restore_backup(file_path: &str, connection_string: &str) -> Result<String, String> {
    let (_tmp_dir, pgsql_path) = extract_pgsql(file_path)?;
    run_pg_restore(&pgsql_path, connection_string)
    // _tmp_dir is dropped here, cleaning up the temp directory
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_extract_pgsql_file_not_found() {
        let result = extract_pgsql("/nonexistent/file.tar.gz");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("File not found"));
    }

    #[test]
    fn test_extract_pgsql_invalid_archive() {
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(b"not a tar.gz file").unwrap();
        let result = extract_pgsql(tmp.path().to_str().unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to extract tar.gz"));
    }

    #[test]
    fn test_extract_pgsql_valid_archive() {
        // Create a tar.gz with a .pgsql file inside
        let tmp_dir = TempDir::new().unwrap();
        let tar_gz_path = tmp_dir.path().join("backup.tar.gz");

        // Create a .pgsql file to add to the archive
        let pgsql_content = b"-- pg_restore test data";

        // Build tar.gz in memory
        let tar_gz_file = std::fs::File::create(&tar_gz_path).unwrap();
        let encoder = flate2::write::GzEncoder::new(tar_gz_file, flate2::Compression::default());
        let mut tar_builder = tar::Builder::new(encoder);

        let mut header = tar::Header::new_gnu();
        header.set_size(pgsql_content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();

        tar_builder
            .append_data(&mut header, "backup.pgsql", &pgsql_content[..])
            .unwrap();
        let encoder = tar_builder.into_inner().unwrap();
        encoder.finish().unwrap();

        let (extracted_dir, pgsql_path) = extract_pgsql(tar_gz_path.to_str().unwrap()).unwrap();
        assert!(pgsql_path.exists());
        assert_eq!(pgsql_path.extension().unwrap(), "pgsql");

        // Verify content
        let content = std::fs::read_to_string(&pgsql_path).unwrap();
        assert_eq!(content, "-- pg_restore test data");

        drop(extracted_dir);
    }

    #[test]
    fn test_extract_pgsql_no_pgsql_in_archive() {
        let tmp_dir = TempDir::new().unwrap();
        let tar_gz_path = tmp_dir.path().join("no_pgsql.tar.gz");

        let tar_gz_file = std::fs::File::create(&tar_gz_path).unwrap();
        let encoder = flate2::write::GzEncoder::new(tar_gz_file, flate2::Compression::default());
        let mut tar_builder = tar::Builder::new(encoder);

        let content = b"just a text file";
        let mut header = tar::Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();

        tar_builder
            .append_data(&mut header, "readme.txt", &content[..])
            .unwrap();
        let encoder = tar_builder.into_inner().unwrap();
        encoder.finish().unwrap();

        let result = extract_pgsql(tar_gz_path.to_str().unwrap());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No .pgsql file found"));
    }

    #[test]
    fn test_find_pgsql_file_empty_dir() {
        let tmp_dir = TempDir::new().unwrap();
        let result = find_pgsql_file(tmp_dir.path());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No .pgsql file found"));
    }

    #[test]
    fn test_run_pg_restore_not_found() {
        // Use a bogus command path to simulate pg_restore not found
        let tmp = NamedTempFile::new().unwrap();
        let result = run_pg_restore(tmp.path(), "postgresql://localhost/testdb");
        // This test depends on whether pg_restore is installed
        // If not installed, it should return a "not found" error
        // If installed, it will fail because the connection is invalid
        assert!(result.is_err());
    }

    #[test]
    fn test_restore_backup_file_not_found() {
        let result = restore_backup("/nonexistent/backup.tar.gz", "postgresql://localhost/testdb");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("File not found"));
    }
}
