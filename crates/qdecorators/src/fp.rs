//! fp — functional primitives for the kitchen-sink boundary: pipe, tap,
//! fixed-point iteration, composition, and deterministic memoization.
//! Nothing fancy, everything deterministic.

/// Pipe — `value.pipe(f)` reads left-to-right, the way the wire reads.
pub trait Pipe: Sized {
    fn pipe<R>(self, f: impl FnOnce(Self) -> R) -> R {
        f(self)
    }
}
impl<T> Pipe for T {}

/// Tap — a side channel that returns the value unchanged: `.tap(|v| …)`
/// for immutable peeks, `.tap_mut(|v| …)` for repairs in place.
pub trait Tap: Sized {
    fn tap(self, f: impl FnOnce(&Self)) -> Self {
        f(&self);
        self
    }
    fn tap_mut(mut self, f: impl FnOnce(&mut Self)) -> Self {
        f(&mut self);
        self
    }
}
impl<T> Tap for T {}

/// seq — fixed-point iteration: `seq(x0, 8, f)` folds `f` eight times.
/// The engine's tick loops, expressed as a value: the world folds from
/// the seed, round after round, in the same order every replay.
pub fn seq<T: Clone>(mut x: T, rounds: usize, f: impl Fn(&T) -> T) -> T {
    for _ in 0..rounds {
        x = f(&x);
    }
    x
}

/// compose — `compose(f, g)(x) == f(g(x))`.
pub fn compose<A, B, C>(f: impl Fn(B) -> C, g: impl Fn(A) -> B) -> impl Fn(A) -> C {
    move |a| f(g(a))
}

/// memoize1 — deterministic memoization over one argument: the same
/// input, the same value, never recomputed — the kit's answer to
/// repeatable costs. The cache is internal, so the function stays pure
/// from the caller's point of view.
pub fn memoize1<K: std::hash::Hash + Eq + Clone, V: Clone>(
    f: impl FnMut(K) -> V,
) -> impl FnMut(K) -> V {
    let mut cache: std::collections::HashMap<K, V> = std::collections::HashMap::new();
    let mut f = f;
    move |k| {
        if let Some(v) = cache.get(&k) {
            return v.clone();
        }
        let v = f(k.clone());
        cache.insert(k, v.clone());
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipe_reads_left_to_right() {
        assert_eq!(5u32.pipe(|x| x + 1).pipe(|x| x * 2), 12);
    }

    #[test]
    fn tap_repairs_in_place() {
        let v = vec![1, 2, 3].tap_mut(|v| v.push(4));
        assert_eq!(v, vec![1, 2, 3, 4]);
        let peeked = 7u32.tap(|x| {
            let _ = x;
        });
        assert_eq!(peeked, 7);
    }

    #[test]
    fn seq_folds_a_fixed_number_of_rounds() {
        // the classic deterministic fold: x_{n+1} = (x_n · 3 + 1) mod 7
        let x8 = seq(1u32, 8, |x| (x * 3 + 1) % 7);
        assert_eq!(x8, seq(1u32, 8, |x| (x * 3 + 1) % 7));
        assert_eq!(x8, 6); // pinned: the fold's eighth round
    }

    #[test]
    fn compose_applies_inside_out() {
        let f = compose(|x: i32| x * 2, |x: i32| x + 1);
        assert_eq!(f(4), 10);
    }

    #[test]
    fn memoize_is_a_pure_function_with_a_cache() {
        // the call counter lives in the same scope as the cache — the
        // borrow-checker wants exclusivity while the closure holds &mut
        let mut calls = 0u32;
        let once = {
            let mut heavy = memoize1(|k: u32| {
                calls += 1;
                k * k
            });
            heavy(4);
            heavy(4);
            drop(heavy);
            calls
        };
        assert_eq!(once, 1, "the second hit skips the body");
        let mut calls2 = 0u32;
        let twice = {
            let mut heavy = memoize1(|k: u32| {
                calls2 += 1;
                k * k
            });
            heavy(4);
            heavy(5);
            drop(heavy);
            let n = calls2;
            assert_eq!(n, 2, "a fresh key recomputes");
            n
        };
        assert_eq!(twice, 2);
    }
}
