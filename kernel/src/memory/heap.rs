const HEAP_TREE_START: usize = 4 * 1024 * 1024 * 1024;
/// The base heap address. The first 4GiB is identity mapped, so we put the heap
/// straight above. We place the info needed for the heap (block tree) above _that_, so the heap
/// only begins after that.
///
/// Should be aligned to 64 bytes _at least_.
// We add 1 because the size of the blocks in tree is a power of two - 1 and we want to be aligned
const HEAP_START: usize = HEAP_TREE_START + mem::size_of::<[Block; BLOCKS_IN_TREE]>() + 1;

use core::alloc::{GlobalAlloc, Layout, Opaque};
use core::{iter, cmp, mem, f32};
use core::ptr::{self, Unique};
use core::ops::{Deref, DerefMut};
use spin::{Once, Mutex};
use util;
use super::paging::{PAGE_TABLES, Page, PageSize, EntryFlags};

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
            // Do round up division, so we do (x + y - 1) / y instead of x / y.
            let pages_to_map = (mem::size_of::<[Block; BLOCKS_IN_TREE]>() + 4096 - 1) / 4096;

            for page in 0..pages_to_map {
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
}

unsafe impl GlobalAlloc for Heap {
    unsafe fn alloc(&self, layout: Layout) -> *mut Opaque {
        let mut tree = self.tree.wait().unwrap().lock();

        let order = order(layout.size());

        if order > MAX_ORDER {
            0 as *mut _
        } else {
            if let Some(ptr) = tree.allocate(order) {
                let ptr = (ptr as usize + HEAP_START) as *mut _;

                // Map ptr page if it isn't already
                let page_tables = PAGE_TABLES; // This is needed, apparently
                let mut page_tables = page_tables.lock();

                let mapped = page_tables
                    .walk_page_table(
                        Page::containing_address(ptr as usize, PageSize::Kib4)
                    ).is_some();

                if !mapped {
                    page_tables.map(
                        Page::containing_address(ptr as usize, PageSize::Kib4),
                        EntryFlags::WRITABLE,
                    );
                }

                ptr
            } else {
                0 as *mut _
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut Opaque, layout: Layout) {
        if !ptr.is_null() {
            let order = order(layout.size());
            let ptr = ptr as usize - HEAP_START;

            self.tree.wait().unwrap().lock().deallocate(ptr as *mut _, order);
        }
    }

    unsafe fn realloc(&self, ptr: *mut Opaque, layout: Layout, new_size: usize) -> *mut Opaque {
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
