//! A simple bitmap allocator used to allocate memory for the buddy allocator

#[cfg(test)]
use std::boxed::Box;
use core::{mem, ptr::{self, Unique}};
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use spin::{Once, Mutex};
use bit_field::BitField;
use super::{physical_allocator::BLOCKS_IN_TREE, buddy_allocator::Block};

pub static BOOTSTRAP_HEAP: BootstrapHeap = BootstrapHeap(Once::new());

/// A holding struct for the bootstrap heap.
pub struct BootstrapHeap(Once<BootstrapAllocator<[Block; BLOCKS_IN_TREE]>>);

impl BootstrapHeap {
    /// Allocates a zeroed object. Panics if bootstrap heap is not initialized
    #[cfg(not(test))]
    pub unsafe fn allocate(&self) -> Option<BootstrapHeapBox<[Block; BLOCKS_IN_TREE]>> {
        self.0.wait().unwrap().allocate()
    }

    /// Initialises the bootstrap heap with a begin address.
    pub unsafe fn init_unchecked(&self, address: usize) {
        self.0.call_once(|| BootstrapAllocator::new_unchecked(address));
    }

    /// Get the end address of the bootstrap heap. Inclusive. Panics if uninitialized
    pub fn end(&self) -> usize {
        self.0.wait().unwrap().start() as usize +
            BootstrapAllocator::<[Block; BLOCKS_IN_TREE]>::space_taken()
    }
}

/// A bitmap heap/physmem allocator to bootstrap the buddy allocator since it requires a
/// (relative to how much the stack should be used for) large amount of memory.
#[derive(Debug)]
pub struct BootstrapAllocator<T> {
    start_addr: usize,
    bitmap: Mutex<u8>,
    _phantom: PhantomData<T>,
}

impl<T> BootstrapAllocator<T> {
    pub const fn space_taken() -> usize {
        mem::size_of::<T>() * 8
    }

    pub fn start(&self) -> *mut T {
        self.start_addr as *mut T
    }

    /// Create an allocator with a start address of `start`. UB if not enough space given to the
    /// allocator (could overwrite other memory) or if the start ptr is not well aligned.
    pub const fn new_unchecked(start: usize) -> Self {
        BootstrapAllocator {
            start_addr: start,
            bitmap: Mutex::new(0),
            _phantom: PhantomData,
        }
    }

    /// Set a block to used or not at an index
    #[inline]
    fn set_used(&self, index: usize, used: bool) {
        let bit = index % 8;
        self.bitmap.lock().set_bit(bit, used);
    }

    /// Allocate an object and return the address if there is space
    unsafe fn allocate<'a>(&'a self) -> Option<BootstrapHeapBox<'a, T>> {
        for bit in 0..8 {
            let mut byte = self.bitmap.lock();

            if !byte.get_bit(bit) {
                byte.set_bit(bit, true);

                let ptr = Unique::new_unchecked(self.start().offset((bit) as isize));
                return Some(BootstrapHeapBox { ptr, allocator: self });
            }
        }

        None
    }

    /// Deallocate a heap box. Must be only called in the `Drop` impl of `BootstrapHeapBox`.
    unsafe fn deallocate(&self, obj: &BootstrapHeapBox<T>) {
        let addr_in_heap = obj.ptr.as_ptr() as usize - self.start_addr;
        let index = addr_in_heap / mem::size_of::<T>();

        self.set_used(index, false);
    }
}

pub struct BootstrapHeapBox<'a, T: 'a> {
    ptr: Unique<T>,
    allocator: &'a BootstrapAllocator<T>,
}

impl<'a, T> PartialEq for BootstrapHeapBox<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        ptr::eq(self.ptr.as_ptr() as *const _, other.ptr.as_ptr() as *const _)
    }
}

impl<'a, T> Eq for BootstrapHeapBox<'a, T> {}

impl<'a, T> Deref for BootstrapHeapBox<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }
}

impl<'a, T> DerefMut for BootstrapHeapBox<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.ptr.as_mut() }
    }
}

impl<'a, T> Drop for BootstrapHeapBox<'a, T> {
    fn drop(&mut self) {
        unsafe { self.allocator.deallocate(self); }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn setup_heap() -> *mut u8{
        let area = unsafe { Box::new(
            mem::zeroed::<[u8; BootstrapAllocator::<u8>::space_taken()]>()
        )};
        let start = Box::into_raw(area);
        start as *mut u8
    }

    fn teardown_heap(ptr: *mut u8) {
        unsafe { drop(Box::from_raw(ptr)) };
    }

    #[test]
    fn test_bitmap_dealloc() {
        let ptr = setup_heap();
        let bitmap = BootstrapAllocator::<u8>::new_unchecked(ptr as usize);

        let heap_box = unsafe { bitmap.allocate().unwrap() };
        let old_ptr = heap_box.ptr;
        drop(heap_box);
        assert!(ptr::eq(
            unsafe { bitmap.allocate().unwrap().ptr.as_ptr() },
            old_ptr.as_ptr()
        ));

        teardown_heap(ptr);
    }


    #[test]
    fn test_bitmap_allocate_with_free() {
        let ptr = setup_heap();

        let bitmap = BootstrapAllocator::<u8>::new_unchecked(ptr as usize);

        assert_eq!(
            unsafe { bitmap.allocate().unwrap().ptr.as_ptr() },
            ptr as *mut _
        );

        teardown_heap(ptr);
    }

    #[test]
    fn test_bitmap_allocate() {
        use ::std::vec::Vec;

        let ptr = setup_heap();
        let bitmap = BootstrapAllocator::<u8>::new_unchecked(ptr as usize);
        let mut v = Vec::with_capacity(8);

        for i in 0..8 {
            let obj = unsafe { bitmap.allocate().unwrap() };
            assert!(ptr::eq(obj.ptr.as_ptr(), (ptr as *mut u8).wrapping_offset(i as isize)));
            v.push(obj); // Stop it from being dropped
        }

        teardown_heap(ptr);
    }

    #[test]
    fn test_bitmap_allocate_runs_out() {
        use ::std::vec::Vec;

        let ptr = setup_heap();

        let bitmap = BootstrapAllocator::<u8>::new_unchecked(ptr as usize);
        let mut v = Vec::with_capacity(8);

        for _ in 0..8 {
            let addr = unsafe { bitmap.allocate().unwrap() };
            v.push(addr); // Stop it from being dropped
        }

        assert!(unsafe { bitmap.allocate() } == None);

        teardown_heap(ptr);
    }
}
