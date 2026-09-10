//! decorate — the decorator macros: raw templates and determinism, made
//! kit-shaped.

/// raw_fmt! — format! over a raw template, double-hash by convention.
///
/// The footgun this codebase keeps stepping on: a raw string with a
/// single-hash opener terminates at the first quote-hash pair that
/// follows it — so any markup carrying an attribute quote next to a hex
/// color (a stroke in the family palette) is silently cut in half, and
/// rustc's parser answers with a "prefix unknown" that has nothing to
/// do with the truth. The convention: **every raw template uses the
/// double-hash delimiters** — a lone quote-hash pair is inert inside a
/// double-hash string, and the triple sequence does not occur in
/// rendered markup. This macro names the convention; the
/// `check-raw-strings` guard (e2e + CI) fails on any regression to
/// single-hash.
///
/// ```
/// let svg = qdecorators::raw_fmt!(r##"<rect fill="#0b0f19" width="{}"/>"##, 42);
/// assert!(svg.contains("width=\"42\""));
/// ```
#[macro_export]
macro_rules! raw_fmt {
    ($tpl:literal $(, $($arg:expr),+)?) => {
        format!($tpl $(, $($arg),+)?)
    };
}

/// assert_deterministic! — the determinism decorator: runs the same
/// expression twice and demands byte-identical results. The engine's
/// replay discipline, as one macro — the fold is admissible or it is
/// not, and "not" fails right here, with both halves shown.
///
/// ```
/// let a = [1, 2, 3];
/// qdecorators::assert_deterministic!(a.iter().sum::<i32>());
/// ```
#[macro_export]
macro_rules! assert_deterministic {
    ($expr:expr) => {{
        let first = $expr;
        let second = $expr;
        assert_eq!(
            first, second,
            "the world is not replayable: the same seed, different folds"
        );
        first
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_raw_template_convention_holds_even_with_hash_quotes() {
        let svg = raw_fmt!(r##"<rect fill="#0b0f19" width="{}"/>"##, 42);
        assert_eq!(svg, r##"<rect fill="#0b0f19" width="42"/>"##);
    }

    #[test]
    fn the_determinism_decorator_enforces_replayability() {
        let v = assert_deterministic!([1i32, 2, 3].iter().map(|x| x * x).sum::<i32>());
        assert_eq!(v, 14);
    }

    #[test]
    #[should_panic(expected = "not replayable")]
    fn the_decorator_panics_on_drift() {
        // a counter captures the drift the engine forbids
        let mut n = 0u32;
        assert_deterministic!({
            n += 1;
            n
        });
    }
}
