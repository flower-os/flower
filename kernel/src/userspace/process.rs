use core::ptr;
use core::sync::atomic::{Ordering, AtomicU64};
use ccl::dhashmap::DHashMap;
use crate::gdt::GDT;
use crate::util::{self, RNG};
use crate::memory::paging::*;

use super::*;

lazy_static! {
    pub static ref PROCESSES: DHashMap<ProcessId, Process> = DHashMap::with_nonce(RNG.next());
}

pub static NEXT_PID: AtomicU64 = AtomicU64::new(0);

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ProcessId(u64);

impl ProcessId {
    pub fn next() -> Self {
        let next_pid = NEXT_PID.fetch_add(1, Ordering::Relaxed);

        assert!(
            next_pid < u64::max_value(),
            "Ran out of process ids. This should never happen"
        );

        ProcessId(next_pid)
    }
}

#[derive(Debug)]
pub struct Process {
    pub page_tables: InactivePageMap,
    stack_ptr: usize,
    instruction_ptr: usize,
    new: bool,
}

impl Process {
    pub unsafe fn spawn(instruction_ptr: usize) -> ProcessId {
        let page_tables = new_process_page_tables();

        let process = Process {
            page_tables,
            stack_ptr: STACK_TOP as usize,
            instruction_ptr,
            new: true,
        };

        let pid = process::ProcessId::next();
        let page_tables = process.page_tables.clone();
        process::PROCESSES.insert(pid, process);

        pid
    }

    pub fn run(&mut self) -> ! {
        ACTIVE_PAGE_TABLES.lock().switch(&self.page_tables);

        if self.new {
            unsafe { self.setup(); }
            self.new = false;
        }

        unsafe {
            super::jump::jump_usermode(self.stack_ptr, self.instruction_ptr)
        }
    }

    /// Sets up the process for it to be run for the first time. Assumes that the page tables have
    /// been switched to the process's AND that the processor is in ring0.
    unsafe fn setup(&mut self) {
        // Set up user stack
        let stack_top = Page::containing_address(STACK_TOP, PageSize::Kib4);
        let stack_bottom = stack_top - INITIAL_STACK_SIZE_PAGES;
        trace!("stack bottom = 0x{:x}", stack_bottom.start_address().unwrap());

        ACTIVE_PAGE_TABLES.lock().map_range(
            stack_bottom..=stack_top,
            EntryFlags::WRITABLE | EntryFlags::USER_ACCESSIBLE | EntryFlags::NO_EXECUTE,
            InvalidateTlb::NoInvalidate,
            ZeroPage::Zero,
        );

        let stack_size = INITIAL_STACK_SIZE_PAGES * 0x1000;
    }
}
