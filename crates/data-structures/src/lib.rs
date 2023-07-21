#![warn(clippy::pedantic)]
#![feature(allocator_api)]

pub mod graph;
pub mod linked_list;

#[cfg(test)]
mod test {
    pub use std::sync::Arc;
    use std::{
        alloc::{Allocator, Global},
        fmt::Debug,
        sync::atomic::{AtomicIsize, Ordering},
    };

    #[derive(Default)]
    pub struct TestingAllocator<A: Allocator = Global> {
        allocator: A,
    }

    unsafe impl<A: Allocator> Allocator for TestingAllocator<A> {
        fn allocate(
            &self,
            layout: std::alloc::Layout,
        ) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
            println!("allocating {} bytes...", layout.size());
            self.allocator.allocate(layout)
        }

        unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, layout: std::alloc::Layout) {
            println!("deallocating {} bytes...", layout.size());
            self.allocator.deallocate(ptr, layout);
        }
    }

    impl<A: Allocator> TestingAllocator<A> {
        pub fn new_in(allocator: A) -> Self {
            Self { allocator }
        }
    }

    impl TestingAllocator<Global> {
        pub fn new() -> Self {
            Self::new_in(Global)
        }
    }

    #[derive(Debug, Default)]
    pub struct AllocationCounter {
        allocations: AtomicIsize,
    }

    impl AllocationCounter {
        pub fn new() -> Arc<Self> {
            Arc::new(Self::default())
        }

        pub fn count<T: Debug>(self: Arc<Self>, data: T) -> CounterGuard<T> {
            self.allocations.fetch_add(1, Ordering::SeqCst);
            CounterGuard {
                data,
                counter: Arc::clone(&self),
            }
        }
    }

    impl Drop for AllocationCounter {
        fn drop(&mut self) {
            assert_eq!(self.allocations.load(Ordering::SeqCst), 0);
        }
    }

    #[derive(Debug, Clone)]
    pub struct CounterGuard<T: Debug> {
        pub data: T,
        counter: Arc<AllocationCounter>,
    }

    impl<T: Debug> Drop for CounterGuard<T> {
        fn drop(&mut self) {
            println!("dropping {:?}", self.data);
            self.counter.allocations.fetch_sub(1, Ordering::SeqCst);
        }
    }
}
