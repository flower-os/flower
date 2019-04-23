use crate::memory::paging::Page;

/// A bump allocator for kernel stacks. There is no guard page.
pub struct StackAllocator {
    base: Page,
    capacity: usize,
    /// Stack size in 4kib pages
    stack_size_pages: usize,
    current: usize,
}

impl StackAllocator {
    pub fn new(base: Page, capacity: usize, stack_size: usize) -> StackAllocator {
        base.start_address().expect("Page requires size");

        StackAllocator {
            base,
            capacity,
            stack_size_pages: stack_size,
            current: 0,
        }
    }

    pub fn alloc(&mut self) -> Option<*const u8> {
        if self.current >= self.capacity {
            return None;
        }

        let addr = self.base.start_address().unwrap() + (self.current * (self.stack_size_pages << 12));
        self.current += 1;

        Some(addr as *const u8)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::memory::paging::PageSize;

    #[test]
    fn test_stack_alloc() {
        let mut allocator = StackAllocator::new(Page::containing_address(0, PageSize::Kib4), 10, 1);

        assert_eq!(allocator.alloc(), Some(0 as *const u8));
        assert!(allocator.alloc() != allocator.alloc());
        assert_eq!(allocator.alloc(), Some((3 << 12) as *const u8));
    }

    #[test]
    fn test_stack_alloc_runs_out() {
        let mut allocator = StackAllocator::new(Page::containing_address(0, PageSize::Kib4), 10, 1);

        for _ in 0..10 {
            allocator.alloc();
        }

        assert_eq!(allocator.alloc(), None);
    }
}
