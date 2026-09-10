// retopo.rs — automated mesh normalization & validation hooks.
//
// Layer-1 mesh intake: validate GLB/GLTF assets before they reach the
// staging directory, and report the numbers a retopo pass cares about
// (meshes, primitives, accessors). Hooks, not a full retopologizer —
// the heavy geometry work belongs to the export seams (Blender/Unity).

use serde_json::Value;

/// The mesh report a retopo hook consumes.
#[derive(Debug, Clone, PartialEq)]
pub struct MeshInfo {
    pub format: String,
    pub meshes: u64,
    pub primitives: u64,
    pub accessors: u64,
}

/// inspect_mesh — validate the container and count the geometry.
/// GLB: magic "glTF" + version 2. GLTF: JSON with meshes/accessors.
pub fn inspect_mesh(path: &std::path::Path) -> Result<MeshInfo, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    if bytes.len() >= 4 && &bytes[..4] == b"glTF" {
        if bytes.len() < 12 {
            return Err("glb too short".into());
        }
        let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        if version != 2 {
            return Err(format!("unsupported glb version {version}"));
        }
        return Ok(MeshInfo {
            format: "glb".into(),
            meshes: 0, // the JSON chunk would be parsed for exact counts
            primitives: 0,
            accessors: 0,
        });
    }
    let text = std::str::from_utf8(&bytes).map_err(|_| "not utf-8 gltf".to_string())?;
    let v: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let meshes = v
        .get("meshes")
        .and_then(|m| m.as_array())
        .map(|a| a.len())
        .unwrap_or(0) as u64;
    let primitives = v
        .get("meshes")
        .and_then(|m| m.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m.get("primitives").and_then(|p| p.as_array()))
                .map(|p| p.len() as u64)
                .sum()
        })
        .unwrap_or(0);
    let accessors = v
        .get("accessors")
        .and_then(|a| a.as_array())
        .map(|a| a.len())
        .unwrap_or(0) as u64;
    Ok(MeshInfo {
        format: "gltf".into(),
        meshes,
        primitives,
        accessors,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspects_gltf_json() {
        let dir = std::env::temp_dir().join(format!("pipeline-retopo-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("wolf.gltf");
        std::fs::write(
            &p,
            r#"{"meshes":[{"primitives":[{"attributes":{}},{"attributes":{}}]},{"primitives":[{}]}],"accessors":[{"type":"VEC3"}]}"#,
        )
        .unwrap();
        let info = inspect_mesh(&p).unwrap();
        assert_eq!(info.format, "gltf");
        assert_eq!(info.meshes, 2);
        assert_eq!(info.primitives, 3);
        assert_eq!(info.accessors, 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_bad_glb() {
        let dir = std::env::temp_dir().join(format!("pipeline-retopo-bad-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("bad.glb");
        std::fs::write(&p, b"glTF\x07\x00\x00\x00xxxx").unwrap();
        assert!(inspect_mesh(&p).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
