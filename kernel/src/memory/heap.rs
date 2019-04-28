/// The base heap address.
pub const HEAP_START: usize = 0xffffffff40000000;

use core::alloc::{GlobalAlloc, Layout};
use core::{iter, mem};
use core::ptr::Unique;
use core::ops::{Deref, DerefMut};
use spin::{Once, Mutex};
use super::paging::{ACTIVE_PAGE_TABLES, Page, PageSize, EntryFlags, FreeMemory, InvalidateTlb};
use crate::memory::{buddy_allocator, paging::PhysicalAddress};
use crate::util;
// use ...::Block // <-- this one comes from the macro invocation below

const BASE_ORDER: u8 = 6;

buddy_allocator_bitmap_tree!(LEVEL_COUNT = 25, BASE_ORDER = BASE_ORDER);

/// Wrapper that just impls deref for a Unique.
///
/// # Safety
///
/// Safe if the Unique is valid.
struct DerefPtr<T>(Unique<T>);

impl<T> DerefPtr<T> {
    const fn new(unique: Unique<T>) -> Self {
        DerefPtr(unique)
    }
}

impl<T> Deref for DerefPtr<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.0.as_ref() }
    }
}

impl<T> DerefMut for DerefPtr<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.0.as_mut() }
    }
}

pub struct Heap {
    tree: Once<Mutex<Tree<DerefPtr<[Block; BLOCKS_IN_TREE]>>>>,
}

impl Heap {
    pub const fn new() -> Self {
        Heap { tree: Once::new() }
    }
    
    /// Initializes the heap. Required for it to be usable, otherwise all of its methods will panic.
    ///
    /// # Unsafety
    ///
    /// Unsafe because `heap_tree_start` needs to be correct (unused) and well-aligned (currently
    /// non applicable as Block is a u8).
    pub unsafe fn init(&self, heap_tree_start: usize) -> usize {
        self.tree.call_once(|| {
            // Get the next page up from the given heap start
            let heap_tree_start = ((heap_tree_start / 4096) + 1) * 4096;

            // Map pages for the tree to use for accounting info
            let pages_to_map = util::round_up_divide(
                mem::size_of::<[Block; BLOCKS_IN_TREE]>() as u64,
                4096,
            );

            for page in 0..pages_to_map as usize {
                let mut table = ACTIVE_PAGE_TABLES.lock();
                table.map(
                    Page::containing_address(heap_tree_start + (page * 4096), PageSize::Kib4),
                    EntryFlags::WRITABLE | EntryFlags::NO_EXECUTE | EntryFlags::USER_ACCESSIBLE, // TODO
                    InvalidateTlb::Invalidate,
                );
            }

            let tree = Tree::new(
                iter::once(0..(1 << 30 + 1)),
                DerefPtr::new(Unique::new_unchecked(heap_tree_start as *mut _)),
            );

            Mutex::new(tree)
        });

        ((heap_tree_start / 4096) + 1) * 4096
    }

    /// Allocate a block of minimum size of 4096 bytes (rounded to this if smaller) with specific
    /// requirements about where it is to be placed in physical memory.
    ///
    /// Note: `physical_begin_frame` is the frame number of the beginning physical frame to allocate
    /// memory from (i.e address / 4096).
    ///
    /// # Panicking
    ///
    /// Panics if the heap is not initialized.
    ///
    /// # Unsafety
    ///
    /// Unsafe as it remaps pages, which could cause memory unsafety if the heap is not set up
    /// correctly.
    pub unsafe fn alloc_specific(
        &self,
        physical_begin_frame: usize,
        frames: usize,
    ) -> *mut u8 {
        let mut tree = self.tree.wait().expect("Heap not initialized!").lock();
        
        let order = order(frames * 4096);
        if order > MAX_ORDER { return 0 as *mut _; }

        let ptr = tree.allocate(order);

        if ptr.is_none() { return 0 as *mut _; }

        let ptr = (ptr.unwrap() as usize + HEAP_START) as *mut u8;

        // Map pages that must be mapped
        for page in 0..util::round_up_divide(1u64 << (order + BASE_ORDER), 4096) as usize {
            let page_addr = ptr as usize + (page * 4096);
            ACTIVE_PAGE_TABLES.lock().map_to(
                Page::containing_address(page_addr, PageSize::Kib4),
                PhysicalAddress((physical_begin_frame + page) * 4096),
                EntryFlags::WRITABLE | EntryFlags::NO_EXECUTE | EntryFlags::USER_ACCESSIBLE, // TODO
                InvalidateTlb::Invalidate,
            );
        }

        ptr
    }

    /// The `dealloc` counterpart to `alloc_specific`. This function does not free the backing
    /// physical memory.
    /// 
    /// # Panicking
    ///
    /// Panics if the heap is not initialized.
    ///
      /// # Unsafety
    ///
    /// Unsafe as it unmaps pages, which could cause memory unsafety if the heap is not set up
    /// correctly.
    pub unsafe fn dealloc_specific(&self, ptr: *mut u8, frames: usize) {
        if ptr.is_null() || frames == 0 {
            return;
        }

        let order = order(frames * 4096);

        assert!(
            ptr as usize >= HEAP_START &&
                (ptr as usize) < (HEAP_START + (1 << 30)),
            "Heap object {:?} pointer not in heap!",
            ptr,
        );

        let global_ptr = ptr;
        let ptr = ptr as usize - HEAP_START;

        self.tree.wait().expect("Heap not initialized!").lock().deallocate(ptr as *mut _, order);

        // Unmap pages that have were used for this alloc
        for page in 0..util::round_up_divide(1u64 << (order + BASE_ORDER), 4096) as usize {
            let page_addr = global_ptr as usize + (page * 4096);

            ACTIVE_PAGE_TABLES.lock().unmap(
                Page::containing_address(page_addr, PageSize::Kib4),
                FreeMemory::NoFree,
                InvalidateTlb::NoInvalidate,
            );
        }
    }

    pub const fn tree_size() -> usize {
        mem::size_of::<[Block; BLOCKS_IN_TREE]>()
    }
}

unsafe impl GlobalAlloc for Heap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut tree = self.tree.wait().expect("Heap not initialized!").lock();

        let order = order(layout.size());
        if order > MAX_ORDER { return 0 as *mut _ }

        let ptr = tree.allocate(order);
        if ptr.is_none() { return 0 as *mut _ }
        let ptr = (ptr.unwrap() as usize + HEAP_START) as *mut u8;

        // Map pages that have yet to be mapped
        for page in 0..util::round_up_divide(1u64 << (order + BASE_ORDER - 1), 4096) as usize {
            let mut page_tables = ACTIVE_PAGE_TABLES.lock();

            let page_addr = ptr as usize + (page * 4096);

            let mapped = page_tables
                .walk_page_table(
                    Page::containing_address(page_addr, PageSize::Kib4)
                ).is_some();

            if !mapped {
                page_tables.map(
                    Page::containing_address(page_addr, PageSize::Kib4),
                    EntryFlags::WRITABLE | EntryFlags::NO_EXECUTE | EntryFlags::USER_ACCESSIBLE, // TODO
                    InvalidateTlb::NoInvalidate,
                );
            }
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if ptr.is_null() {
            return;
        }

        let order = order(layout.size());

        assert!(
            ptr as usize >= HEAP_START &&
                (ptr as usize) < (HEAP_START + (1 << 30)),
            "Heap object {:?} pointer not in heap!",
            ptr,
        );

        let global_ptr = ptr;
        let ptr = ptr as usize - HEAP_START;

        self.tree.wait().expect("Heap not initialized!").lock().deallocate(ptr as *mut _, order);

        let page_order = 12 - BASE_ORDER; // log2(4096) - base order

        // There will only be pages to unmap which totally contained this allocation if this
        // allocation was larger or equal to the size of a page
        if order < page_order  {
            // Else, we must check if it happened to be on its own page

            let page_base_ptr = ptr & !0xFFF;

            let level = MAX_ORDER - page_order;
            let level_offset = buddy_allocator::blocks_in_tree(level);
            let index = level_offset + (page_base_ptr >> (page_order + BASE_ORDER)) + 1;
            let order_free = self.tree.wait().unwrap().lock().block(index - 1).order_free;

            if order_free == page_order + 1 {
                let global_ptr = page_base_ptr + HEAP_START;

                ACTIVE_PAGE_TABLES.lock().unmap(
                    Page::containing_address(global_ptr, PageSize::Kib4),
                    FreeMemory::Free,
                    InvalidateTlb::NoInvalidate, // TODO invalidate
                );
            }
        } else {
           // Unmap pages that have were only used for this alloc
           for page in 0..util::round_up_divide(1u64 << (order + BASE_ORDER - 1), 4096) as usize {
               let page_addr = global_ptr as usize + (page * 4096);

               ACTIVE_PAGE_TABLES.lock().unmap(
                   Page::containing_address(page_addr, PageSize::Kib4),
                   FreeMemory::Free,
                   InvalidateTlb::NoInvalidate, // TODO invalidate
               );
           }
       }
   }
}

fn order(val: usize) -> u8 {
    if val == 0 {
        return 0;
    }

    let log2 = log2_ceil(val as u64) + 1;

    if log2 > BASE_ORDER {
        log2 - BASE_ORDER
    } else {
        0
    }
}

fn log2_ceil(val: u64) -> u8 {
    let log2 = log2_floor(val);
    if val != (1u64 << log2) {
        log2 + 1
    } else {
        log2
    }
}

fn log2_floor(mut val: u64) -> u8 {
    let mut log2 = 0;
    while val > 1 {
        val >>= 1;
        log2 += 1;
    }
    log2 as u8
}
