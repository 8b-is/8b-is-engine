//! lanes — the two action lanes, L1–L3, and the compute menu (o1-style).
//!
//! The brain outside these walls speaks OpenAI-style: a fast local lane
//! and a root lane, three effort levels, and a compute knob that
//! includes the named `r1` model and o1-style reasoning budgets. This
//! module is the TYPED map of that grid — the fixtures the wrapper
//! (`scripts/lanes.py`) and the dashboards draw from, deterministic so
//! a boot path and a deep path can never be confused.
//!
//! The two lanes:
//! * `ActionLane::Fast` — thunky-lite: the quick, local, read-mostly lane,
//!   **recommended during boot** (the firewall boot, the doctor, the
//!   seed).
//! * `ActionLane::Root` — the root lane: edits, deployments, rewrites.
//!
//! Levels are compute budgets, o1-style: L1 shallow, L2 working, L3
//! deep — each with a reasoning-token budget the OpenAI-compatible
//! call translates into `reasoning_effort`.

use std::fmt;

/// The two action lanes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionLane {
    /// thunky-lite — the fast local lane (recommended during boot)
    Fast,
    /// the root lane (edits, deployments, rewrites)
    Root,
}

impl ActionLane {
    /// The boot recommendation, spelled as a sentence the wrapper can
    /// print verbatim.
    pub fn recommended_during_boot() -> &'static str {
        "Lane::Fast · thunky-lite — the boot's recommended lane"
    }

    /// The lane's canonical OpenAI-compatible model face.
    pub fn model(self) -> &'static str {
        match self {
            ActionLane::Fast => "thunky-lite",
            ActionLane::Root => "root",
        }
    }
}

/// The three effort levels — o1-style compute budgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    L1,
    L2,
    L3,
}

impl Level {
    /// The reasoning-token budget this level buys (o1-style: more
    /// thought, more tokens, better answers at the margin).
    pub fn reasoning_budget(self) -> u32 {
        match self {
            Level::L1 => 1_024,
            Level::L2 => 4_096,
            Level::L3 => 16_384,
        }
    }

    /// The OpenAI-compatible reasoning-effort string for this level.
    pub fn reasoning_effort(self) -> &'static str {
        match self {
            Level::L1 => "low",
            Level::L2 => "medium",
            Level::L3 => "high",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Level::L1 => "L1",
            Level::L2 => "L2",
            Level::L3 => "L3",
        }
    }
}

/// The compute menu: plain, the known `r1` model, or o1-style reasoning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compute {
    Plain,
    /// the known model — `r1`
    R1,
    /// OpenAI-style reasoning compute: a named effort rides along
    O1 {
        effort: ReasoningEffort,
    },
}

/// The o1-style reasoning-effort knob.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReasoningEffort {
    Low,
    Medium,
    High,
}

impl ReasoningEffort {
    pub fn as_str(self) -> &'static str {
        match self {
            ReasoningEffort::Low => "low",
            ReasoningEffort::Medium => "medium",
            ReasoningEffort::High => "high",
        }
    }
}

/// One action plan: a lane, a level, a compute mode, a label.
#[derive(Debug, Clone, Copy)]
pub struct ActionPlan {
    pub lane: ActionLane,
    pub level: Level,
    pub compute: Compute,
    pub label: &'static str,
}

impl ActionPlan {
    /// The boot plan — the default when nothing is argued otherwise.
    pub fn boot() -> Self {
        ActionPlan {
            lane: ActionLane::Fast,
            level: Level::L1,
            compute: Compute::Plain,
            label: "boot · thunky-lite",
        }
    }

    /// to_effort — the level's o1-style budget, translated for the
    /// OpenAI-compatible call.
    pub fn to_effort(&self) -> &'static str {
        if self.compute == Compute::R1 {
            return "r1"; // the known model answers with its own lens
        }
        match self.compute {
            Compute::O1 { effort } => effort.as_str(),
            _ => self.level.reasoning_effort(),
        }
    }
}

impl fmt::Display for ActionPlan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} · {} · compute {}/{}",
            self.label,
            self.level.label(),
            match self.compute {
                Compute::Plain => "plain".to_string(),
                Compute::R1 => "r1".to_string(),
                Compute::O1 { effort } => format!("o1:{}", effort.as_str()),
            },
            self.to_effort()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_boot_plan_is_fast_and_shallow() {
        let boot = ActionPlan::boot();
        assert_eq!(boot.lane, ActionLane::Fast);
        assert_eq!(boot.level, Level::L1);
        assert_eq!(boot.compute, Compute::Plain);
        assert_eq!(boot.to_effort(), "low");
        assert!(boot.label.contains("boot"));
    }

    #[test]
    fn levels_scale_the_reasoning_budget() {
        assert!(Level::L1.reasoning_budget() < Level::L2.reasoning_budget());
        assert!(Level::L2.reasoning_budget() < Level::L3.reasoning_budget());
        assert_eq!(Level::L3.reasoning_budget(), 16_384);
    }

    #[test]
    fn r1_is_a_known_compute_face() {
        let plan = ActionPlan {
            lane: ActionLane::Root,
            level: Level::L2,
            compute: Compute::R1,
            label: "root · r1",
        };
        assert_eq!(plan.to_effort(), "r1");
    }

    #[test]
    fn o1_effort_rides_along() {
        let plan = ActionPlan {
            lane: ActionLane::Root,
            level: Level::L3,
            compute: Compute::O1 {
                effort: ReasoningEffort::High,
            },
            label: "root · o1",
        };
        assert_eq!(plan.to_effort(), "high");
        assert!(plan.to_string().contains("o1:high"));
    }

    #[test]
    fn the_two_lanes_never_cross() {
        assert_ne!(ActionLane::Fast, ActionLane::Root);
        assert_ne!(ActionLane::Fast.model(), ActionLane::Root.model());
        assert!(ActionLane::recommended_during_boot().contains("thunky-lite"));
    }
}
