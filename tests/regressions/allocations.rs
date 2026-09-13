use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

thread_local! {
    // A const initializer avoids allocating while the allocator reads the counter.
    static ALLOCATION_COUNT: Cell<Option<usize>> = const { Cell::new(None) };
}

struct CountingAllocator;

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn record_allocation() {
    // Allocation during thread teardown must not panic if TLS is unavailable.
    let _ = ALLOCATION_COUNT.try_with(|count| {
        if let Some(current) = count.get() {
            count.set(Some(current.saturating_add(1)));
        }
    });
}

// SAFETY: Every allocator operation forwards its arguments unchanged to System.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record_allocation();
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        record_allocation();
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        record_allocation();
        unsafe { System.realloc(ptr, layout, new_size) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

/// Count allocation and reallocation calls on this thread during `operation`.
pub fn count_allocations<T>(operation: impl FnOnce() -> T) -> (T, usize) {
    struct Reset;

    impl Drop for Reset {
        fn drop(&mut self) {
            ALLOCATION_COUNT.with(|count| count.set(None));
        }
    }

    ALLOCATION_COUNT.with(|count| {
        assert!(count.get().is_none(), "allocation measurements cannot nest");
        count.set(Some(0));
        let reset = Reset;
        let result = operation();
        let allocations = count.get().unwrap();
        drop(reset);
        (result, allocations)
    })
}
