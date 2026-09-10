//! mq — the core's message passing: a tiny in-process message queue and
//! the locker (the single-writer door, generalized).
//!
//! The engine speaks through queues and one author at a time: `Mq` is a
//! FIFO the actors exchange; `Locker` is the named gate that lets
//! exactly one task through to shared state at any moment.

use std::collections::VecDeque;
use std::sync::{mpsc, Mutex};

/// A tiny FIFO message queue.
#[derive(Debug, Default)]
pub struct Mq<T> {
    inner: VecDeque<T>,
}

impl<T> Mq<T> {
    pub fn new() -> Self {
        Mq {
            inner: VecDeque::new(),
        }
    }

    pub fn push(&mut self, m: T) {
        self.inner.push_back(m);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.inner.pop_front()
    }

    pub fn peek(&self) -> Option<&T> {
        self.inner.front()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// drain — the queue's messages in order, emptying it.
    pub fn drain(&mut self) -> Vec<T> {
        self.inner.drain(..).collect()
    }
}

/// The standard channel pair, named for the house: `Sender` → `Receiver`.
pub fn chan<T>() -> (mpsc::Sender<T>, mpsc::Receiver<T>) {
    mpsc::channel()
}

/// The locker — the single-writer door over shared state: `execute`
/// lets exactly one closure through at a time.
pub struct Locker<T> {
    inner: Mutex<T>,
}

impl<T> Locker<T> {
    pub fn new(value: T) -> Self {
        Locker {
            inner: Mutex::new(value),
        }
    }

    /// execute — take the door, mutate, return the door's verdict.
    pub fn execute<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut guard = self.inner.lock().expect("locker: the door sticks");
        f(&mut guard)
    }

    pub fn try_execute<R>(&self, f: impl FnOnce(&mut T) -> R) -> Option<R> {
        let mut guard = self.inner.try_lock().ok()?;
        Some(f(&mut guard))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_queue_is_fifo() {
        let mut q = Mq::new();
        q.push(1);
        q.push(2);
        q.push(3);
        assert_eq!(q.len(), 3);
        assert_eq!(q.pop(), Some(1));
        assert_eq!(q.peek(), Some(&2));
        assert_eq!(q.drain(), vec![2, 3]);
        assert!(q.is_empty());
    }

    #[test]
    fn the_channel_hands_off() {
        let (tx, rx) = chan();
        tx.send("the world folds".to_string()).unwrap();
        assert_eq!(rx.recv().unwrap(), "the world folds");
    }

    #[test]
    fn the_locker_serializes() {
        let door = std::sync::Arc::new(Locker::new(0u64));
        let mut handles = Vec::new();
        for _ in 0..8 {
            let door = door.clone();
            handles.push(std::thread::spawn(move || {
                for _ in 0..250 {
                    door.execute(|v| *v += 1);
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(door.execute(|v| *v), 2000, "8 × 250 through one door");
    }
}
