// stager.rs — validation, checksums, and staging into assets/staged/.
//
// The stager is the layer-1 attestation for files: every staged asset is
// copied via temp-then-rename, SHA256-checksummed, and recorded in a
// manifest. Corruption is detectable, not detectable-by-accident.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;

/// One staged asset and its attestation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StagedEntry {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
}

/// The stage manifest — the admitted inventory of assets/staged/.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StageManifest {
    pub version: u32,
    pub brief: String,
    pub seed_line: String,
    pub entries: Vec<StagedEntry>,
}

/// sha256_file — the checksum of a file's bytes, hex-encoded.
pub fn sha256_file(path: &Path) -> std::io::Result<String> {
    let mut f = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex(&hasher.finalize()))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// stage_file — copy `src` into `dst_dir` atomically (temp file + rename),
/// then verify the copy's checksum. Returns the attestation entry.
pub fn stage_file(src: &Path, dst_dir: &Path) -> std::io::Result<StagedEntry> {
    std::fs::create_dir_all(dst_dir)?;
    let name = src
        .file_name()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "no file name"))?;
    let dst = dst_dir.join(name);
    let tmp = dst_dir.join(format!(".tmp-{}", name.to_string_lossy()));

    let sum_before = sha256_file(src)?;
    std::fs::copy(src, &tmp)?;
    std::fs::rename(&tmp, &dst)?;
    let sum_after = sha256_file(&dst)?;
    if sum_before != sum_after {
        let _ = std::fs::remove_file(&dst);
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("checksum mismatch staging {name:?}"),
        ));
    }
    Ok(StagedEntry {
        path: name.to_string_lossy().to_string(),
        sha256: sum_after,
        bytes: std::fs::metadata(&dst)?.len(),
    })
}

/// validate_staged — re-checksum every entry against the manifest.
/// Returns the first corruption found, or Ok(()) when the stage is intact.
pub fn validate_staged(dir: &Path, manifest: &StageManifest) -> Result<(), String> {
    for entry in &manifest.entries {
        let path = dir.join(&entry.path);
        if !path.exists() {
            return Err(format!("missing staged asset: {}", entry.path));
        }
        let sum = sha256_file(&path).map_err(|e| e.to_string())?;
        if sum != entry.sha256 {
            return Err(format!("corrupted staged asset: {}", entry.path));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmpdir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("pipeline-test-{tag}-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn staging_preserves_bytes_and_detects_corruption() {
        let src_dir = tmpdir("src");
        let dst_dir = tmpdir("dst");
        let src = src_dir.join("terrain.ron");
        std::fs::write(&src, b"{\"board\":[0,1,-1]}\n").unwrap();

        let entry = stage_file(&src, &dst_dir).unwrap();
        assert_eq!(entry.bytes, 19);
        assert_eq!(entry.sha256, sha256_file(&dst_dir.join("terrain.ron")).unwrap());

        let manifest = StageManifest {
            version: 1,
            brief: "the pink tent".into(),
            seed_line: "0".into(),
            entries: vec![entry.clone()],
        };
        assert_eq!(validate_staged(&dst_dir, &manifest), Ok(()));

        // corrupt the staged copy — the attestation must fail
        std::fs::write(dst_dir.join("terrain.ron"), b"{\"board\":[0,1,1]}\n").unwrap();
        let err = validate_staged(&dst_dir, &manifest).unwrap_err();
        assert!(err.contains("corrupted"), "got: {err}");

        let _ = std::fs::remove_dir_all(&src_dir);
        let _ = std::fs::remove_dir_all(&dst_dir);
    }
}
