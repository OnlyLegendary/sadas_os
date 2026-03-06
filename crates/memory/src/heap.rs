use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use core::sync::atomic::{AtomicBool, Ordering};

struct HeapState {
    start: usize,
    size: usize,
    offset: usize,
    initialized: bool,
}

impl HeapState {
    const fn new() -> Self {
        Self {
            start: 0,
            size: 0,
            offset: 0,
            initialized: false,
        }
    }

    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.size = size;
        self.offset = 0;
        self.initialized = true;
    }

    fn alloc(&mut self, layout: Layout) -> *mut u8 {
        if !self.initialized {
            return null_mut();
        }

        let alloc_start = align_up(self.start + self.offset, layout.align());
        let alloc_end = alloc_start.saturating_add(layout.size());

        if alloc_end > self.start.saturating_add(self.size) {
            return null_mut();
        }

        self.offset = alloc_end - self.start;
        alloc_start as *mut u8
    }
}

pub struct LockedHeap {
    lock: AtomicBool,
    state: core::cell::UnsafeCell<HeapState>,
}

unsafe impl Sync for LockedHeap {}

impl LockedHeap {
    pub const fn new() -> Self {
        Self {
            lock: AtomicBool::new(false),
            state: core::cell::UnsafeCell::new(HeapState::new()),
        }
    }

    pub fn init(&self, start: usize, size: usize) {
        self.with_lock(|s| s.init(start, size));
    }

    fn with_lock<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut HeapState) -> R,
    {
        while self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {}

        let result = f(unsafe { &mut *self.state.get() });
        self.lock.store(false, Ordering::Release);
        result
    }
}

unsafe impl GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.with_lock(|s| s.alloc(layout))
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Bump allocator: deallocation is a no-op.
    }
}

pub static GLOBAL_ALLOCATOR: LockedHeap = LockedHeap::new();

pub fn init_global_heap(start: usize, size: usize) {
    GLOBAL_ALLOCATOR.init(start, size);
}

fn align_up(value: usize, align: usize) -> usize {
    debug_assert!(align.is_power_of_two());
    (value + align - 1) & !(align - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bump_allocates_aligned_chunks() {
        static HEAP: LockedHeap = LockedHeap::new();
        HEAP.init(0x1000, 0x1000);

        let a = unsafe { HEAP.alloc(Layout::from_size_align(8, 8).expect("layout")) } as usize;
        let b = unsafe { HEAP.alloc(Layout::from_size_align(16, 16).expect("layout")) } as usize;

        assert_eq!(a % 8, 0);
        assert_eq!(b % 16, 0);
        assert!(b >= a + 8);
    }
}
