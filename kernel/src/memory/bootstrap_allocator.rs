//! A simple bitmap allocator used to allocate memory for the buddy allocator

use spin::Mutex;
#[cfg(test)]
use std::boxed::Box;
use core::mem;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut, Range};
use bit_field::BitField;
use super::buddy_allocator::{Block, BLOCKS_IN_TREE};

/// How many 8 heap objects to allocate at maximum.
const OBJECTS8_NUMBER: usize = 32; // 32 gives us 256GiB while taking 8MiB of memory

type MutexOption<T> = Mutex<Option<T>>;
pub static BOOTSTRAP_ALLOCATOR: BootstrapHeap = BootstrapHeap(Mutex::new(None));

pub struct BootstrapHeap(pub Mutex<Option<BootstrapAllocator<[Block; BLOCKS_IN_TREE]>>>);

impl BootstrapHeap {
    /// Panics if bootstrap heap is not initialized
    pub unsafe fn allocate_zeroed(&self) -> Option<BootstrapHeapBox<[Block; BLOCKS_IN_TREE]>> {
        self.0.lock().as_mut().unwrap().allocate_zeroed()
    }

    /// Initialises the bootstrap heap with a begin address
    pub unsafe fn init_unchecked(&self, address: usize) {
        *self.0.lock() = Some(BootstrapAllocator::new_unchecked(address));
    }
}

/// A bitmap heap/physmem allocator to bootstrap the buddy allocator since it requires a
/// (relative to how much the stack should be used for) large amount of memory.
pub struct BootstrapAllocator<T> {
    start_addr: usize,
    frames8: [Objects8; OBJECTS8_NUMBER],
    _phantom: PhantomData<T>,
}

/// A bitmap consisting of 8 frames
#[derive(Copy, Clone)]
struct Objects8(u8);

impl<T> BootstrapAllocator<T> {
    pub const fn space_taken() -> usize {
        mem::size_of::<T>() * OBJECTS8_NUMBER
    }

    fn start(&self) -> *mut T {
        self.start_addr as *mut T
    }

    /// Create an allocator with a start address of `start`. UB if not enough space given to the
    /// allocator (could overwrite other memory) or if the start ptr is not well aligned.
    pub fn new_unchecked(start: usize) -> Self {
        BootstrapAllocator {
            start_addr: start,
            frames8: [Objects8(0); OBJECTS8_NUMBER],
            _phantom: PhantomData,
        }
    }

    /// Set a block to used or not at an index
    fn set_used(&mut self, index: usize, used: bool) {
        let frames8_index = index / 8;
        let bit = index % 8;
        self.frames8[frames8_index].0.set_bit(bit, used);
    }

    // TODO do we need this
    /// Set a range of blocks to used or not used
    fn set_used_range(&mut self, range: Range<usize>, used: bool) {
        // Sort of inefficient but it's probably fine
        for (frames8_index, bit) in range.map(|i| (i / 8, i % 8)) {
            self.frames8[frames8_index].0.set_bit(bit, used);
        }
    }

    /// Get if a block is used at an index
    fn get_used(&mut self, index: usize) -> bool {
        let frames8_index = index / 8;
        let bit = index % 8;
        self.frames8[frames8_index].0.get_bit(bit)
    }

    /// Allocate an object of zeroes and return the address if there is space
    pub unsafe fn allocate_zeroed(&mut self) -> Option<BootstrapHeapBox<T>> {
        for frames8_index in 0..OBJECTS8_NUMBER {
            for bit in 0..8 {
                if !self.frames8[frames8_index].0.get_bit(bit) {
                    self.frames8[frames8_index].0.set_bit(bit, true);

                    let ptr = self.start().offset((frames8_index * 8 + bit) as isize);
                    *ptr = mem::zeroed();

                    return Some(BootstrapHeapBox {
                        ptr,
                    });
                }
            }
        }

        None
    }

    /// Allocate an object and return the address if there is space
    pub fn allocate(&mut self, obj: T) -> Option<BootstrapHeapBox<T>> {
        for frames8_index in 0..OBJECTS8_NUMBER {
            for bit in 0..8 {
                if !self.frames8[frames8_index].0.get_bit(bit) {
                    self.frames8[frames8_index].0.set_bit(bit, true);

                    let ptr = unsafe { self.start().offset((frames8_index * 8 + bit) as isize) };
                    unsafe { *ptr = obj };

                    return Some(BootstrapHeapBox {
                        ptr,
                    });
                }
            }
        }

        None
    }
}

#[derive(Eq, PartialEq, Debug)]
pub struct BootstrapHeapBox<T> {
    ptr: *mut T,
}

impl<T> Deref for BootstrapHeapBox<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.ptr }
    }
}

impl<T> DerefMut for BootstrapHeapBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ptr }
    }
}

impl<T> Drop for BootstrapHeapBox<T> {
    fn drop(&mut self) {

    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_bitmap_used() {
        let area = unsafe { Box::new(
            mem::zeroed::<[u8; BootstrapAllocator::<u8>::space_taken()]>()
        )};
        let start = Box::into_raw(area);

        let mut bitmap = BootstrapAllocator::<u8>::new_unchecked(start as usize);

        bitmap.set_used(9, true);
        bitmap.set_used(8, true); // TODO
        
        bitmap.set_used(0, true);
        assert!(bitmap.get_used(9));
        assert!(bitmap.get_used(0));
        assert!(!bitmap.get_used(1));

        drop(unsafe { Box::from_raw(start) });
    }

    #[test]
    fn test_bitmap_set_range() {
        let area = unsafe { Box::new(
            mem::zeroed::<[u8; BootstrapAllocator::<u8>::space_taken()]>()
        )};
        let start = Box::into_raw(area);

        let mut bitmap = BootstrapAllocator::<u8>::new_unchecked(start as usize);

        bitmap.set_used_range(0..10, true);
        for bit in 0..10 {
            assert!(bitmap.get_used(bit));
        }

        assert!(!bitmap.get_used(10));

        drop(unsafe { Box::from_raw(start) });
    }

    #[test]
    fn test_bitmap_allocate_with_free() {
        let area = unsafe { Box::new(
            mem::zeroed::<[u8; BootstrapAllocator::<u8>::space_taken()]>()
        )};
        let start = Box::into_raw(area);

        let mut bitmap = BootstrapAllocator::<u8>::new_unchecked(start as usize);

        assert_eq!(bitmap.allocate(1).unwrap().ptr, start as *mut _);
        assert_eq!(
            bitmap.allocate(1).unwrap().ptr,
            (start as *mut u8).wrapping_offset(1 as isize)
        );

        drop(unsafe { Box::from_raw(start) });
    }

    #[test]
    fn test_bitmap_allocate_no_free() {
        let area = unsafe { Box::new(
            mem::zeroed::<[u8; BootstrapAllocator::<u8>::space_taken()]>()
        )};
        let start = Box::into_raw(area);

        let mut bitmap = BootstrapAllocator::<u8>::new_unchecked(start as usize);

        bitmap.set_used_range(0..(OBJECTS8_NUMBER * 8), true);
        let address = bitmap.allocate(1);
        assert_eq!(address, None);

        drop(unsafe { Box::from_raw(start) });
    }

    #[test]
    fn test_bitmap_allocate() {
        let area = unsafe { Box::new(
            mem::zeroed::<[u8; BootstrapAllocator::<u8>::space_taken()]>()
        )};
        let start = Box::into_raw(area);

        let mut bitmap = BootstrapAllocator::<u8>::new_unchecked(start as usize);

        for i in 0..(OBJECTS8_NUMBER * 8) {
            let obj = bitmap.allocate(1).unwrap();
            assert_eq!(obj.ptr, (start as *mut u8).wrapping_offset(i as isize));
            assert_eq!(*obj, 1);
        }

        drop(unsafe { Box::from_raw(start) });
    }

    #[test]
    fn test_bitmap_allocate_zeroed() {
        let area = unsafe { Box::new(
            mem::zeroed::<[u8; BootstrapAllocator::<u8>::space_taken()]>()
        )};
        let start = Box::into_raw(area);

        let mut bitmap = BootstrapAllocator::<u8>::new_unchecked(start as usize);

        for i in 0..(OBJECTS8_NUMBER * 8) {
            let obj = unsafe { bitmap.allocate_zeroed().unwrap() };
            assert_eq!(obj.ptr, (start as *mut u8).wrapping_offset(i as isize));
            assert_eq!(*obj, 0);
        }

        drop(unsafe { Box::from_raw(start) });
    }

    #[test]
    fn test_bitmap_allocate_runs_out() {
        use ::std::vec::Vec;

        let area = unsafe { Box::new(
            mem::zeroed::<[u8; BootstrapAllocator::<u8>::space_taken()]>()
        )};
        let start = Box::into_raw(area);

        let mut bitmap = BootstrapAllocator::<u8>::new_unchecked(start as usize);

        for _ in 0..(OBJECTS8_NUMBER * 8) {
            bitmap.allocate(0).unwrap();
        }

        assert_eq!(bitmap.allocate(0), None);

        drop(unsafe { Box::from_raw(start) });
    }
}