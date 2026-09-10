//! dag.rs — the pipeline's own graph: layer-1's stages as an ultra-graph.
//!
//! The pipeline describes itself: every stage is a node, every bond is a
//! tri-state edge (`+1` deterministic/attested, `-1` the external lane —
//! the LLM adapter, non-deterministic when wired, `0` the optional art
//! lanes). One kind, fixed order, deterministic JSON — the same replay
//! discipline the rest of the engine lives by, applied to the engine's
//! own ingestion.

use serde_json::json;

/// The layer-1 stage DAG, as a deterministic tri-state graph.
pub fn stage_dag_json(label: &str) -> String {
    let nodes: [&str; 7] = [
        "gdd-text",
        "deterministic-expander",
        "llm-expander",
        "zone-manifest",
        "concept-art",
        "vision-qa",
        "stage-manifest",
    ];
    // (from, to, w): +1 authoritative, -1 the external lane, 0 optional
    let edges: [(usize, usize, i8); 8] = [
        (0, 1, 1),  // gdd → deterministic: the default truth
        (0, 2, -1), // gdd → llm: the adapter lane only when wired
        (1, 3, 1),  // deterministic → manifest: attested by construction
        (2, 3, -1), // llm → manifest: attested only by the operator's word
        (3, 4, 0),  // manifest → art: the optional, dark-when-absent lane
        (4, 5, 0),  // art → vision QA: the eye, when the art lane renders
        (3, 6, 1),  // manifest → stage: the first attested asset
        (4, 6, 0),  // art → stage: staged alongside when it rendered
    ];
    serde_json::to_string_pretty(&json!({
        "kind": "pipeline-dag",
        "label": label,
        "nodes": nodes,
        "edges": edges
            .iter()
            .map(|&(a, b, w)| vec![a as i64, b as i64, w as i64])
            .collect::<Vec<_>>(),
    }))
    .expect("dag json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_dag_is_deterministic_and_ternary() {
        let a = stage_dag_json("layer-1");
        let b = stage_dag_json("layer-1");
        assert_eq!(a, b, "the pipeline's self-graph must be replayable");
        let v: serde_json::Value = serde_json::from_str(&a).unwrap();
        assert_eq!(v["kind"], "pipeline-dag");
        assert_eq!(v["nodes"].as_array().unwrap().len(), 7);
        for e in v["edges"].as_array().unwrap() {
            let w = e.as_array().unwrap()[2].as_i64().unwrap();
            assert!((-1..=1).contains(&w), "tri-state bond only");
        }
    }

    #[test]
    fn every_stage_receives_a_bond() {
        let v: serde_json::Value = serde_json::from_str(&stage_dag_json("x")).unwrap();
        let edges = v["edges"].as_array().unwrap();
        let mut reached = std::collections::HashSet::new();
        for e in edges {
            let arr = e.as_array().unwrap();
            reached.insert(arr[1].as_i64().unwrap());
        }
        assert_eq!(reached.len(), 6, "the manifest plus every consumer reached");
    }
}
