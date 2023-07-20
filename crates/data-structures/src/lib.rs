#![warn(clippy::pedantic)]
#![feature(allocator_api)]

pub mod graph;
pub mod linked_list;

#[cfg(test)]
mod test {
    use std::{
        alloc::{Allocator, Global},
        fmt::Debug,
    };

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

    pub struct DropWrapper<T: Debug>(pub T);

    impl<T: Debug> Drop for DropWrapper<T> {
        fn drop(&mut self) {
            println!("dropping {:?}...", self.0);
        }
    }
}
