#![no_std]

use core::sync::atomic::{AtomicU64, Ordering};

pub struct EntropyPool {
    state: AtomicU64,
}

impl EntropyPool {
    pub const fn new(seed: u64) -> Self {
        Self {
            state: AtomicU64::new(seed),
        }
    }

    pub fn stir(&self, input: u64) {
        let mixed = input.rotate_left(13).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        self.state.fetch_xor(mixed, Ordering::Relaxed);
    }

    pub fn random_u64(&self) -> u64 {
        let mut current = self.state.load(Ordering::Relaxed);
        loop {
            let next = current
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            match self
                .state
                .compare_exchange_weak(current, next, Ordering::Relaxed, Ordering::Relaxed)
            {
                Ok(_) => return next,
                Err(observed) => current = observed,
            }
        }
    }
}

pub static ENTROPY: EntropyPool = EntropyPool::new(0x5341_4441_5300_0001);
