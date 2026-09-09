//! pipeline — Layer 1: ingestion and staging for the 8b-is engine.
//!
//! Text in, typed-and-checksummed assets out. The orchestrator runs the
//! stages sequentially (parse → optional art → stage) on Tokio without
//! blocking the runtime, and every stage is an attested inscription:
//! determinism by default, the LLM/adapter path only when the operator
//! wires it, checksums on everything that reaches `assets/staged/`.

pub mod asset_gen;
pub mod gdd_parser;
pub mod retopo;
pub mod stager;

use gdd_parser::{CommandExpander, DeterministicExpander, ExpandError, GddExpander, ZoneManifest};
use std::path::{Path, PathBuf};
use stager::{stage_file, StageManifest, StagedEntry};

fn env(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

/// The pipeline's configuration — env-only, never a committed secret.
///
/// * `VAKED_PIPELINE_LLM_CMD` — a colon-free command line (space-split)
///   that reads GDD text on stdin and prints strict JSON on stdout. When
///   unset, the deterministic expander runs (the default, and the truth).
/// * `VAKED_MLX_SIDECAR_DIR` — the mlx-sidecar uv project (concept art +
///   vision QA lanes).
/// * `VAKED_PIPELINE_STAGE_DIR` — where staged assets land.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub llm_cmd: Option<Vec<String>>,
    pub art_lanes: asset_gen::ArtLanes,
    pub stage_dir: PathBuf,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

impl PipelineConfig {
    pub fn from_env() -> Self {
        let sidecar = env(
            "VAKED_MLX_SIDECAR_DIR",
            "../vaked-lsp/mlx-sidecar", // relative to crates/pipeline
        );
        let stage = env("VAKED_PIPELINE_STAGE_DIR", "assets/staged");
        let llm_cmd = std::env::var("VAKED_PIPELINE_LLM_CMD")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.split_whitespace().map(str::to_string).collect());
        Self {
            llm_cmd,
            art_lanes: asset_gen::ArtLanes::new(PathBuf::from(sidecar)),
            stage_dir: PathBuf::from(stage),
        }
    }

    /// the expander this config selects — deterministic unless wired
    pub fn expander(&self) -> Box<dyn GddExpander> {
        match &self.llm_cmd {
            Some(cmd) => Box::new(CommandExpander { cmd: cmd.clone() }),
            None => Box::new(DeterministicExpander),
        }
    }
}

/// The artifacts one pipeline run produces.
#[derive(Debug)]
pub struct PipelineOutput {
    pub manifest: ZoneManifest,
    pub ron_path: PathBuf,
    pub entry: StagedEntry,
    pub concept_art: Option<PathBuf>,
    pub qa: Option<String>,
}

/// run_pipeline — the async orchestrator: parse → (art → QA) → stage.
/// Sequential by design: each stage consumes the previous stage's
/// attestation; nothing is generated before the brief is admitted.
pub async fn run_pipeline(
    gdd_text: &str,
    cfg: &PipelineConfig,
    with_art: bool,
) -> Result<PipelineOutput, String> {
    // stage 1: parse — the deterministic expander is pure CPU and the
    // command adapter already blocks on its child; no runtime handoff needed
    let expander = cfg.expander();
    let manifest = expander.expand(gdd_text).map_err(|e: ExpandError| e.to_string())?;

    // stage 2-3: concept art + the eye (the local lanes; dark when absent)
    let mut concept_art = None;
    let mut qa = None;
    if with_art {
        if cfg.art_lanes.available() {
            let art_dir = PathBuf::from("assets/generated");
            match cfg
                .art_lanes
                .concept_art(&manifest.brief, &art_dir)
                .await
            {
                Ok(png) => {
                    let verdict = cfg
                        .art_lanes
                        .qa_render(&png, "Is this concept art admissible for the 8b-is world? One sentence, then one improvement.")
                        .await
                        .ok();
                    concept_art = Some(png);
                    qa = verdict;
                }
                Err(e) => eprintln!("  art lane dark: {e}"),
            }
        } else {
            eprintln!("  art lane dark: mlx-sidecar not found");
        }
    }

    // stage 4: stage the manifest itself as the first attested asset
    std::fs::create_dir_all(&cfg.stage_dir).map_err(|e| e.to_string())?;
    let ron_path = std::env::temp_dir().join(format!(
        "zone-{}.ron",
        &manifest.seed_line[..8.min(manifest.seed_line.len())]
    ));
    let ron_text = ron::ser::to_string_pretty(&manifest, ron::ser::PrettyConfig::default())
        .map_err(|e| e.to_string())?;
    std::fs::write(&ron_path, ron_text).map_err(|e| e.to_string())?;
    let entry = stage_file(&ron_path, &cfg.stage_dir).map_err(|e| e.to_string())?;

    // stage the concept art alongside, when it rendered
    if let Some(png) = &concept_art {
        let _ = stage_file(png, &cfg.stage_dir);
    }

    // the stage manifest: the admitted inventory
    let mut entries = vec![entry.clone()];
    if let Ok(read) = std::fs::read_dir(&cfg.stage_dir) {
        for f in read.flatten() {
            let e = stage_file(&f.path(), &cfg.stage_dir).ok();
            if let Some(e) = e {
                if !entries.iter().any(|x| x.path == e.path) {
                    entries.push(e);
                }
            }
        }
    }
    let sm = StageManifest {
        version: 1,
        brief: manifest.brief.clone(),
        seed_line: manifest.seed_line.clone(),
        entries,
    };
    let sm_path = cfg.stage_dir.join("stage-manifest.ron");
    let sm_text = ron::ser::to_string_pretty(&sm, ron::ser::PrettyConfig::default())
        .map_err(|e| e.to_string())?;
    std::fs::write(&sm_path, sm_text).map_err(|e| e.to_string())?;

    Ok(PipelineOutput {
        manifest,
        ron_path,
        entry,
        concept_art,
        qa,
    })
}

/// verify_staged — re-checksum the stage directory against its manifest.
pub fn verify_staged(cfg: &PipelineConfig) -> Result<(), String> {
    let manifest = read_manifest(&cfg.stage_dir)?;
    stager::validate_staged(&cfg.stage_dir, &manifest)
}

/// read_manifest — load the stage manifest from a directory.
pub fn read_manifest(dir: &Path) -> Result<StageManifest, String> {
    let text = std::fs::read_to_string(dir.join("stage-manifest.ron")).map_err(|e| e.to_string())?;
    ron::from_str(&text).map_err(|e| e.to_string())
}
