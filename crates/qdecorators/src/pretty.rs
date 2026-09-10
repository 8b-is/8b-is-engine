//! pretty — Python-traceback-style diagnostics, with the fix as a
//! one-line sed.
//!
//! The constellation's errors are rare, and when they come they should
//! arrive the way Python's trade dress arrives: a file, a line, a hunk
//! with a caret, a message that says what HAPPENED, and a repair that
//! says what TO DO — human-readable AND machine-runnable: the
//! `sed_command()` of a `Fix` is a literal one-liner the operator can
//! paste. The compiler's "try this" — made a first-class citizen.

/// The repair: a one-line patch (sed syntax) plus the reasoning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fix {
    /// A literal sed script, e.g. `s|r#"|r##"|g` — pasteable.
    pub sed: String,
    /// Why this repair is right.
    pub note: String,
}

impl Fix {
    pub fn new(sed: impl Into<String>, note: impl Into<String>) -> Self {
        Fix {
            sed: sed.into(),
            note: note.into(),
        }
    }

    /// sed_command — the full pasteable repair: `sed -i '<script>' <file>`.
    pub fn sed_command(&self, file: &str) -> String {
        format!("sed -i '{}' {}", self.sed, file)
    }
}

/// The diagnostic: everything the operator needs, nothing they must dig
/// for.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub kind: String,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<usize>,
    pub hunk: Option<String>,
    pub fix: Option<Fix>,
}

impl Diagnostic {
    pub fn new(kind: impl Into<String>, message: impl Into<String>) -> Self {
        Diagnostic {
            kind: kind.into(),
            message: message.into(),
            file: None,
            line: None,
            hunk: None,
            fix: None,
        }
    }

    /// at — the file+line the world was at when it refused.
    pub fn at(mut self, file: impl Into<String>, line: usize) -> Self {
        self.file = Some(file.into());
        self.line = Some(line);
        self
    }

    /// with_hunk — the source line the caret points into (Python's
    /// traceback's best trade dress, kept).
    pub fn with_hunk(mut self, hunk: impl Into<String>) -> Self {
        self.hunk = Some(hunk.into());
        self
    }

    /// with_fix — attach the one-line sed repair.
    pub fn with_fix(mut self, fix: Fix) -> Self {
        self.fix = Some(fix);
        self
    }

    /// render — the pretty traceback: kind, location, hunk + caret,
    /// the message, and (when present) the sed repair.
    pub fn render(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("── {} ", self.kind));
        match (&self.file, self.line) {
            (Some(f), Some(l)) => out.push_str(&format!("· {}:{}", f, l)),
            (Some(f), None) => out.push_str(&format!("· {}", f)),
            _ => out.push_str("· the world refused"),
        }
        out.push('\n');
        if let (Some(h), Some(l)) = (&self.hunk, self.line) {
            out.push_str(&format!("  {}\n", h));
            let pad = h
                .chars()
                .take(l.saturating_sub(1))
                .map(|_| ' ')
                .collect::<String>();
            out.push_str(&format!("  {}^\n", pad));
        }
        out.push_str(&format!("  {}\n", self.message));
        if let Some(fix) = &self.fix {
            out.push_str(&format!("  fix: {}\n", fix.sed));
            if let Some(file) = &self.file {
                out.push_str(&format!("  paste: {}\n", fix.sed_command(file)));
            }
            out.push_str(&format!("  why: {}\n", fix.note));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pretty_render_is_a_traceback_with_a_repair() {
        let d = Diagnostic::new("verify", "an asset drifted from its checksum")
            .at("assets/staged/stage-manifest.ron", 12)
            .with_hunk("        sha256: \"c40eeeb65c00…\"")
            .with_fix(Fix::new(
                "s|c40eeeb65c00.*|c40eeeb65c01|",
                "re-attest the staged file",
            ));
        let text = d.render();
        assert!(text.contains("── verify · assets/staged/stage-manifest.ron:12"));
        assert!(text.contains("^"));
        assert!(text.contains("an asset drifted from its checksum"));
        assert!(text.contains("fix: s|c40eeeb65c00.*|c40eeeb65c01|"));
        assert!(text.contains(
            "paste: sed -i 's|c40eeeb65c00.*|c40eeeb65c01|' assets/staged/stage-manifest.ron"
        ));
    }

    #[test]
    fn the_sed_command_pastes_cleanly() {
        let f = Fix::new(
            "s|r#\"|r##\"|g",
            "the double-hash convention closes the footgun",
        );
        assert_eq!(
            f.sed_command("crates/x.rs"),
            "sed -i 's|r#\"|r##\"|g' crates/x.rs"
        );
    }

    #[test]
    fn missing_fields_stay_honest() {
        let d = Diagnostic::new("pipeline", "the art lane is dark");
        let t = d.render();
        assert!(t.contains("the world refused"));
        assert!(!t.contains("fix:"));
    }
}
