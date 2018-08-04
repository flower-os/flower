const HEAP_TREE_START: usize = 4 * 1024 * 1024 * 1024;
/// The base heap address. The first 4GiB is identity mapped, so we put the heap
/// straight above. We place the info needed for the heap (block tree) above _that_, so the heap
/// only begins after that.
///
/// Should be aligned to 64 bytes _at least_.
// We add 1 because the size of the blocks in tree is a power of two - 1 and we want to be aligned
const HEAP_START: usize = HEAP_TREE_START + mem::size_of::<[Block; BLOCKS_IN_TREE]>() + 1;

use core::alloc::{GlobalAlloc, Layout};
use core::{iter, cmp, mem};
use core::ptr::{self, Unique};
use core::ops::{Deref, DerefMut};
use spin::{Once, Mutex};
use super::paging::{PAGE_TABLES, Page, PageSize, EntryFlags};
use memory::paging::PhysicalAddress;
use util;
// use ...::Block // <-- this one comes from the macro invocation below

buddy_allocator_bitmap_tree!(LEVEL_COUNT = 25, BASE_ORDER = 6);

/// Wrapper that just impls deref for a Unique.
///
/// # Safety
///
/// Safe if the Unique is valid.
struct DerefPtr<T>(Unique<T>);

impl<T> DerefPtr<T> {
    const unsafe fn new(unique: Unique<T>) -> Self {
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

    pub fn init(&self) {
        self.tree.call_once(|| {
            // Map pages for the tree to use for accounting info
            let pages_to_map = util::round_up_divide(
                mem::size_of::<[Block; BLOCKS_IN_TREE]>() as u64,
                4096
            );

            for page in 0..pages_to_map as usize {
                PAGE_TABLES.lock().map(
                    Page::containing_address(HEAP_TREE_START + (page * 4096), PageSize::Kib4),
                    EntryFlags::WRITABLE,
                );
            }

            let tree = unsafe {
                Tree::new(
                    iter::once(0..(1 << 30 + 1)),
                    DerefPtr::new(Unique::new_unchecked(HEAP_TREE_START as *mut _)),
                )
            };

            Mutex::new(tree)
        });
    }

    /// Allocate a block of minimum size of 4096 bytes (rounded to this if smaller) with specific
    /// requirements about where it is to be placed in physical memory.
    ///
    /// Note: `physical_begin_frame` is the frame number of the beginning physical frame to allocate
    /// memory from (i.e address / 4096).
    pub fn alloc_specific(
        &self,
        physical_begin_frame: usize,
        frames: usize,
    ) -> *mut u8 {
        let mut tree = self.tree.wait().unwrap().lock();

        let order = order(frames * 4096);
        if order > MAX_ORDER { return 0 as *mut _ }

        let ptr = tree.allocate(order);
        if ptr.is_none() { return 0 as *mut _ }
        let ptr = (ptr.unwrap() as usize + HEAP_START) as *mut u8;

        // Map pages
        for page in 0..(1 << order) / 4096 {
            let page_addr = ptr as usize + (page * 4096);
            PAGE_TABLES.lock().map_to(
                Page::containing_address(page_addr, PageSize::Kib4),
                PhysicalAddress((physical_begin_frame + page) * 4096),
                EntryFlags::WRITABLE,
            );
        }

        ptr
    }
}

unsafe impl GlobalAlloc for Heap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut tree = self.tree.wait().unwrap().lock();

        let order = order(layout.size());
        if order > MAX_ORDER { return 0 as *mut _ }

        let ptr = tree.allocate(order);
        if ptr.is_none() { return 0 as *mut _ }
        let ptr = (ptr.unwrap() as usize + HEAP_START) as *mut u8;

        // Map pages that have yet to be mapped
        // 6 is base order so `1 << (order + 6)`
        for page in 0..util::round_up_divide(1u64 << (order + 6), 4096) as usize {
            let mut page_tables = PAGE_TABLES.lock();

            let page_addr = ptr as usize + (page * 4096);

            let mapped = page_tables
                .walk_page_table(
                    Page::containing_address(page_addr, PageSize::Kib4)
                ).is_some();

            if !mapped {
                page_tables.map(
                    Page::containing_address(page_addr, PageSize::Kib4),
                    EntryFlags::WRITABLE,
                );
            }
        }

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if !ptr.is_null() {
            let order = order(layout.size());
            let ptr = ptr as usize - HEAP_START;

            self.tree.wait().unwrap().lock().deallocate(ptr as *mut _, order);

            // There will only be pages to unmap which totally contained this allocation if this
            // allocation was larger or equal to the size of a page
            if order < 12 - 6  { // log2(4096) - base order
                return;
            }

            // Unmap pages that have were only used for this alloc
            // 6 is base order so `1 << (order + 6)`
            for page in 0..util::round_up_divide(1u64 << (order + 6), 4096) as usize {
                let page_addr = ptr as usize + (page * 4096);

                PAGE_TABLES.lock().unmap(Page::containing_address(page_addr, PageSize::Kib4));
            }
        }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let old_order = order(layout.size());
        let new_order =  order(new_size);

        // See if the size is still the same order. If so, do nothing
        if old_order == new_order {
            return ptr;
        }

        let new_layout = Layout::from_size_align_unchecked(new_size, layout.align());
        let new_ptr = self.alloc(new_layout);
        if !new_ptr.is_null() {
            ptr::copy_nonoverlapping(
                ptr as *const u8,
                new_ptr as *mut u8,
                cmp::min(layout.size(), new_size),
            );
            self.dealloc(ptr, layout);
        }

        new_ptr
    }
}

/// Calculates the integer log2 of the given input
fn order(i: usize) -> u8 {
    let mut i = i;
    let mut o = 0;
    while i > 0 {
        i >>= 1;
        o += 1;
    }
    o
}
