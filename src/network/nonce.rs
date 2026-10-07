use std::sync::atomic::{AtomicU64, Ordering};

pub struct NonceTracker {
    current: AtomicU64,
}

impl NonceTracker {
    pub fn new(start: u64) -> Self {
        NonceTracker {
            current: AtomicU64::new(start),
        }
    }

    pub fn peek(&self) -> u64 {
        self.current.load(Ordering::SeqCst)
    }

    pub fn take(&self) -> u64 {
        self.current.fetch_add(1, Ordering::SeqCst)
    }

    pub fn reset(&self, value: u64) {
        self.current.store(value, Ordering::SeqCst);
    }

    pub fn sync_forward(&self, observed: u64) {
        let mut now = self.current.load(Ordering::SeqCst);
        while observed > now {
            match self.current.compare_exchange(
                now,
                observed,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => return,
                Err(found) => now = found,
            }
        }
    }
}
