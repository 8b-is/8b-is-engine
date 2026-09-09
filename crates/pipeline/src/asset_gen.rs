// asset_gen.rs — the local asset-generation lanes, wrapped.
//
// The engine's own model lanes (the mlx-sidecar: FLUX.2-klein diffuser for
// concept art, Qwen2.5-VL vision for the QA gate) as subprocess wrappers —
// the same `uv run --project <sidecar> python sidecar.py` shape the
// umbrella MCP uses. The lane stays dark when the sidecar is missing.

use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

/// Where the local lanes live (the mlx-sidecar uv project).
#[derive(Debug, Clone)]
pub struct ArtLanes {
    pub sidecar_dir: PathBuf,
}

impl ArtLanes {
    pub fn new(sidecar_dir: PathBuf) -> Self {
        Self { sidecar_dir }
    }

    pub fn available(&self) -> bool {
        self.sidecar_dir.join("sidecar.py").exists()
    }

    async fn run_sidecar(&self, args: &[&str]) -> Result<String, String> {
        if !self.available() {
            return Err("mlx-sidecar missing — the art lane stays dark".into());
        }
        let sidecar = self.sidecar_dir.join("sidecar.py");
        let out = Command::new("uv")
            .arg("run")
            .arg("--project")
            .arg(&self.sidecar_dir)
            .arg("python")
            .arg(&sidecar)
            .args(args)
            .stdin(Stdio::null())
            .output()
            .await
            .map_err(|e| e.to_string())?;
        if !out.status.success() {
            return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
        }
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }

    /// concept_art — FLUX.2-klein renders the brief; returns the PNG path.
    pub async fn concept_art(&self, brief: &str, out_dir: &std::path::Path) -> Result<PathBuf, String> {
        std::fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;
        let text = self
            .run_sidecar(&[
                "image",
                brief,
                "--steps",
                "4",
                "--out",
                &out_dir.to_string_lossy(),
            ])
            .await?;
        // the sidecar prints `wrote <path>` on success — parse it
        let path = text
            .lines()
            .find_map(|l| l.strip_prefix("wrote "))
            .map(PathBuf::from)
            .ok_or_else(|| format!("no output path from diffuser: {text}"))?;
        Ok(path)
    }

    /// qa_render — the vision lane judges a render: admissible?
    pub async fn qa_render(&self, image: &std::path::Path, question: &str) -> Result<String, String> {
        self.run_sidecar(&[
            "vision",
            &image.to_string_lossy(),
            question,
            "--max-tokens",
            "160",
        ])
        .await
    }
}
