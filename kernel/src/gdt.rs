use spin::Once;
use x86_64::structures::tss::TaskStateSegment;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const PANICKING_EXCEPTION_IST_INDEX: u16 = 1;

pub static TSS: Once<TaskStateSegment> = Once::new();

use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};

lazy_static! {
    static ref GDT: Gdt = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(TSS.wait().unwrap()));
        Gdt {
            table: gdt,
            selectors: Selectors { code_selector, tss_selector },
        }
    };
}

struct Gdt {
    table: GlobalDescriptorTable,
    selectors: Selectors,
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::segmentation::set_cs;
    use x86_64::instructions::tables::load_tss;

    GDT.table.load();

    unsafe {
        set_cs(GDT.selectors.code_selector);
        load_tss(GDT.selectors.tss_selector);
    }
}