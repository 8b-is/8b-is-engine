// tick.rs — the fixed-timestep accumulator: the world's clock.
//
// Simulation steps are decoupled from the wall clock — physics and the
// fold run in fixed steps (the zodiac of the engine), while rendering
// happens when it happens. The accumulator banks fractional time and the
// classic max-steps guard keeps a slow frame from spiraling into a death
// loop: a stalled render costs steps, never the world.

/// The fixed-timestep clock.
#[derive(Debug, Clone)]
pub struct Clock {
    /// the fixed step size (seconds), e.g. 1.0/64.0 for 64 Hz sim
    pub fixed: f64,
    /// banked fractional time, in seconds
    pub accumulator: f64,
    /// the last wall-clock reading, for delta computation
    pub last: f64,
    /// max fixed steps per advance (the spiral-of-death guard)
    pub max_steps: u64,
}

impl Clock {
    /// new — a clock with a fixed step and the default 8-step guard.
    pub fn new(fixed: f64) -> Self {
        Clock {
            fixed,
            accumulator: 0.0,
            last: 0.0,
            max_steps: 8,
        }
    }

    /// advance — feed a wall-clock reading; returns how many fixed steps
    /// the simulation must run this time (0 when not enough time has passed,
    /// clamped to max_steps when the frame was huge).
    pub fn advance(&mut self, now: f64) -> u64 {
        let dt = now - self.last;
        self.last = now;
        if dt < 0.0 || !dt.is_finite() {
            return 0;
        }
        self.accumulator = (self.accumulator + dt).min(self.fixed * self.max_steps as f64);
        let steps = (self.accumulator / self.fixed) as u64;
        self.accumulator -= steps as f64 * self.fixed;
        steps.min(self.max_steps)
    }

    /// step — run `f` for each fixed step the clock demands; returns the
    /// number of steps actually run.
    pub fn step<F: FnMut()>(&mut self, now: f64, mut f: F) -> u64 {
        let n = self.advance(now);
        for _ in 0..n {
            f();
        }
        n
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accumulates_fractional_time() {
        let mut c = Clock::new(1.0 / 64.0);
        c.last = 0.0;
        // 1/256 banks; a further 1/256 makes 1/128 banked — still no step;
        // the closing 1/128 completes exactly one 1/64 step
        assert_eq!(c.advance(1.0 / 256.0), 0);
        assert_eq!(c.advance(1.0 / 128.0), 0);
        assert_eq!(c.advance(1.0 / 64.0), 1);
        assert!(c.accumulator < c.fixed);
    }

    #[test]
    fn a_big_delta_clamps_instead_of_spiraling() {
        let mut c = Clock::new(1.0 / 64.0);
        c.last = 0.0;
        // a stalled render for a full second: at most max_steps, not 64
        assert_eq!(c.advance(1.0), c.max_steps);
        assert_eq!(c.advance(0.0).min(c.max_steps), 0);
    }

    #[test]
    fn step_runs_the_simulation_in_fixed_increments() {
        let mut c = Clock::new(1.0 / 64.0);
        c.last = 0.0;
        let mut ticks = 0u64;
        let n = c.step(1.0 / 32.0, || {
            ticks += 1;
        });
        assert_eq!(n, 2, "two 1/64 steps from a 1/32 delta");
        assert_eq!(ticks, 2);
    }
}
