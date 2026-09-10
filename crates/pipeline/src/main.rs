// pipeline — the layer-1 runner (CLI).
//
//   pipeline run "<brief>"            # parse → stage (deterministic)
//   pipeline run file.md --art        # + FLUX concept art + vision QA
//   pipeline verify                   # re-checksum assets/staged
//   pipeline stage <src> [dst]        # stage one file with attestation

use pipeline::{retopo, run_pipeline, verify_staged, PipelineConfig};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cfg = PipelineConfig::from_env();
    match args.first().map(String::as_str) {
        Some("run") => run(&args[1..], &cfg).await,
        Some("verify") => match verify_staged(&cfg) {
            Ok(()) => println!("staged assets intact — every checksum matches"),
            Err(e) => {
                eprintln!("verify failed: {e}");
                std::process::exit(1);
            }
        },
        Some("stage") => {
            if args.len() < 2 {
                eprintln!("usage: pipeline stage <src> [dst-dir]");
                std::process::exit(2);
            }
            let src = std::path::Path::new(&args[1]);
            let dst =
                std::path::Path::new(args.get(2).map(String::as_str).unwrap_or("assets/staged"));
            match pipeline::stager::stage_file(src, dst) {
                Ok(e) => println!(
                    "staged {} ({}B, sha256 {})",
                    e.path,
                    e.bytes,
                    &e.sha256[..16]
                ),
                Err(e) => {
                    eprintln!("stage failed: {e}");
                    std::process::exit(1);
                }
            }
        }
        Some("mesh") => {
            if args.len() < 2 {
                eprintln!("usage: pipeline mesh <file.gltf|file.glb>");
                std::process::exit(2);
            }
            match retopo::inspect_mesh(std::path::Path::new(&args[1])) {
                Ok(info) => println!(
                    "{}: {} meshes, {} primitives, {} accessors",
                    info.format, info.meshes, info.primitives, info.accessors
                ),
                Err(e) => {
                    eprintln!("mesh rejected: {e}");
                    std::process::exit(1);
                }
            }
        }
        Some("graph") => {
            // the pipeline describes itself: the layer-1 stage DAG as a
            // tri-state ultra-graph
            println!("{}", pipeline::dag::stage_dag_json("layer-1 ingestion"));
        }
        _ => {
            eprintln!("usage: pipeline [run <brief-or-file> [--art] | verify | stage <src> [dst] | mesh <file>]");
            std::process::exit(2);
        }
    }
}

async fn run(args: &[String], cfg: &PipelineConfig) {
    let with_art = args.iter().any(|a| a == "--art");
    let input = args.iter().find(|a| a.as_str() != "--art");
    let text = match input {
        Some(path) if std::path::Path::new(path).exists() => match std::fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("cannot read {path}: {e}");
                std::process::exit(1);
            }
        },
        Some(brief) => brief.clone(),
        None => {
            eprintln!("usage: pipeline run <brief-or-file> [--art]");
            std::process::exit(2);
        }
    };

    match run_pipeline(&text, cfg, with_art).await {
        Ok(out) => {
            println!("⟦ layer 1 ⟧ brief: {}", out.manifest.brief);
            println!("  seed:    {}", out.manifest.seed_line);
            println!(
                "  gaia:    weather {} · gravity {} · memory {}",
                out.manifest.gaia.weather, out.manifest.gaia.gravity, out.manifest.gaia.memory
            );
            println!(
                "  board:   {} tiles of {{-1,0,+1}}",
                out.manifest.board.len()
            );
            println!(
                "  fauna:   {}",
                out.manifest
                    .archetypes
                    .iter()
                    .map(|a| a.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            println!(
                "  items:   {}",
                out.manifest
                    .items
                    .iter()
                    .map(|i| i.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            println!(
                "  staged:  {} ({}B, sha256 {}…)",
                out.entry.path,
                out.entry.bytes,
                &out.entry.sha256[..12]
            );
            if let Some(png) = &out.concept_art {
                println!("  art:     {}", png.display());
            }
            if let Some(qa) = &out.qa {
                println!("  eye:     {}", qa.lines().next().unwrap_or(&qa));
            }
        }
        Err(e) => {
            eprintln!("pipeline failed: {e}");
            std::process::exit(1);
        }
    }
}
