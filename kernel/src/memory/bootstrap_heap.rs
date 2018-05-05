//! A simple bitmap allocator used to allocate memory for the buddy allocator

#[cfg(test)]
use std::boxed::Box;
use core::{mem, ptr::{self, Unique}};
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use spin::{RwLock, RwLockReadGuard, Mutex};
use bit_field::BitField;
use super::physical_allocator::{Block, BLOCKS_IN_TREE};

/// How many 8 heap objects to allocate at maximum.
const OBJECTS8_NUMBER: usize = 32; // 32 gives us 256GiB while taking 8MiB of memory

#[cfg(not(test))]
pub static BOOTSTRAP_HEAP: BootstrapHeap = BootstrapHeap(RwLock::new(None));
#[cfg(test)]
lazy_static! {
    pub static ref BOOTSTRAP_HEAP: BootstrapHeap = {
        // We assume the align is the same so we can allocate a u8 array of the space the bootstrap
        // allocator takes up
        assert_eq!(mem::align_of::<Block>(), 1);

        let area = unsafe {
            box mem::uninitialized::<[u8; BootstrapAllocator::<[Block; BLOCKS_IN_TREE]>::space_taken()]>()
        };
        let start = Box::into_raw(area) as usize;

        BootstrapHeap(RwLock::new(Some(BootstrapAllocator::new_unchecked(start))))
    };
}

pub struct BootstrapHeap(pub RwLock<Option<BootstrapAllocator<[Block; BLOCKS_IN_TREE]>>>);

impl BootstrapHeap {
    /// Panics if bootstrap heap is not initialized
    pub unsafe fn allocate_zeroed(&self) -> Option<BootstrapHeapBox<[Block; BLOCKS_IN_TREE]>> {
        AllocatorRef::ReadGuard(self.0.read()).allocate_zeroed()
    }

    /// Initialises the bootstrap heap with a begin address. Panics if already
    /// initialised.
    pub unsafe fn init_unchecked(&self, address: usize) {
        if let Some(_) = *self.0.read() {
            panic!("Bootstrap heap already initialized!");
        }

        *self.0.write() = Some(BootstrapAllocator::new_unchecked(address));
    }
}

/// A bitmap heap/physmem allocator to bootstrap the buddy allocator since it requires a
/// (relative to how much the stack should be used for) large amount of memory.
#[derive(Debug)]
pub struct BootstrapAllocator<T> {
    start_addr: usize,
    objects8: Mutex<[Objects8; OBJECTS8_NUMBER]>,
    _phantom: PhantomData<T>,
}

/// A bitmap consisting of 8 objects
#[derive(Debug, Copy, Clone)]
struct Objects8(u8);

impl<T> BootstrapAllocator<T> {
    #[cfg(test)]
    pub const fn space_taken() -> usize {
        mem::size_of::<T>() * OBJECTS8_NUMBER * 8
    }

    fn start(&self) -> *mut T {
        self.start_addr as *mut T
    }

    /// Create an allocator with a start address of `start`. UB if not enough space given to the
    /// allocator (could overwrite other memory) or if the start ptr is not well aligned.
    pub const fn new_unchecked(start: usize) -> Self {
        BootstrapAllocator {
            start_addr: start,
            objects8: Mutex::new([Objects8(0); OBJECTS8_NUMBER]),
            _phantom: PhantomData,
        }
    }

    /// Set a block to used or not at an index
    #[inline]
    fn set_used(&self, index: usize, used: bool) {
        let objects8_index = index / 8;
        let bit = index % 8;
        self.objects8.lock()[objects8_index].0.set_bit(bit, used);
    }

    /// Allocate an object of zeroes and return the address if there is space
    unsafe fn allocate_zeroed<'a>(self: AllocatorRef<'a, T>) -> Option<BootstrapHeapBox<'a, T>> {
        for objects8_index in 0..OBJECTS8_NUMBER {
            for bit in 0..8 {
                let mut lock = self.objects8.lock();
                let objects8 = &mut lock[objects8_index].0;

                if !objects8.get_bit(bit) {
                    objects8.set_bit(bit, true);
                    drop(objects8);
                    drop(lock);

                    let ptr = self.start().offset((objects8_index * 8 + bit) as isize);
                    *ptr = mem::zeroed();

                    return Some(BootstrapHeapBox {
                        ptr: Unique::new_unchecked(ptr),
                        allocator: self,
                    });
                }
            }
        }

        None
    }

    /// Deallocate a heap box. Must be only called in the `Drop` impl of `BootstrapHeapBox`.
    unsafe fn deallocate(self: &AllocatorRef<T>, obj: &BootstrapHeapBox<T>) {
        let addr_in_heap = obj.ptr.as_ptr() as usize - self.start_addr;
        let index = addr_in_heap / mem::size_of::<T>();

        self.set_used(index, false);
    }
}

pub struct BootstrapHeapBox<'a, T: 'a> {
    ptr: Unique<T>,
    allocator: AllocatorRef<'a, T>,
}

enum AllocatorRef<'a, T: 'a> {
    #[cfg(test)]
    Ref(&'a BootstrapAllocator<T>),
    ReadGuard(RwLockReadGuard<'a, Option<BootstrapAllocator<T>>>),
}

impl<'a, T: 'a> AllocatorRef<'a, T> {
    fn as_ref<'b>(&'a self) -> &'b BootstrapAllocator<T> where 'a: 'b {
        match self {
            #[cfg(test)]
            AllocatorRef::Ref(r) => r,
            AllocatorRef::ReadGuard(r) => &*r.as_ref().unwrap()
        }
    }
}

impl<'a, T: 'a> Deref for AllocatorRef<'a, T> {
    type Target = BootstrapAllocator<T>;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
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

        let heap_box = unsafe { AllocatorRef::Ref(&bitmap).allocate_zeroed().unwrap() };
        let old_ptr = heap_box.ptr;
        drop(heap_box);
        assert!(ptr::eq(
            unsafe { AllocatorRef::Ref(&bitmap).allocate_zeroed().unwrap().ptr.as_ptr() },
            old_ptr.as_ptr()
        ));

        teardown_heap(ptr);
    }


    #[test]
    fn test_bitmap_allocate_with_free() {
        let ptr = setup_heap();

        let bitmap = BootstrapAllocator::<u8>::new_unchecked(ptr as usize);

        assert_eq!(
            unsafe { AllocatorRef::Ref(&bitmap).allocate_zeroed().unwrap().ptr.as_ptr() },
            ptr as *mut _
        );

        teardown_heap(ptr);
    }

    #[test]
    fn test_bitmap_allocate_zeroed() {
        use ::std::vec::Vec;

        let ptr = setup_heap();
        let bitmap = BootstrapAllocator::<u8>::new_unchecked(ptr as usize);
        let mut v = Vec::with_capacity(OBJECTS8_NUMBER * 8);

        for i in 0..(OBJECTS8_NUMBER * 8) {
            let obj = unsafe { AllocatorRef::Ref(&bitmap).allocate_zeroed().unwrap() };
            assert!(ptr::eq(obj.ptr.as_ptr(), (ptr as *mut u8).wrapping_offset(i as isize)));
            assert_eq!(*obj, 0);
            v.push(obj); // Stop it from being dropped
        }

        teardown_heap(ptr);
    }

    #[test]
    fn test_bitmap_allocate_runs_out() {
        use ::std::vec::Vec;

        let ptr = setup_heap();

        let bitmap = BootstrapAllocator::<u8>::new_unchecked(ptr as usize);
        let mut v = Vec::with_capacity(OBJECTS8_NUMBER * 8);

        for _ in 0..(OBJECTS8_NUMBER * 8) {
            let addr = unsafe { AllocatorRef::Ref(&bitmap).allocate_zeroed().unwrap() };
            v.push(addr); // Stop it from being dropped
        }

        assert!(unsafe { AllocatorRef::Ref(&bitmap).allocate_zeroed() } == None);

        teardown_heap(ptr);
    }
}
