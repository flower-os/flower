use core::sync::atomic::{Ordering, AtomicU64};
use ccl::dhashmap::DHashMap;
use crate::util::RNG;
use crate::memory::paging::{self, InactivePageMap, userspace};

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
}

impl Process {
    pub fn new() -> Self {
        let page_tables = paging::userspace::map_new_process();

        Process {
            page_tables,
        }
    }
}
