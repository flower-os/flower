use core::cell::RefCell;
use spin::{Once, Mutex};
use x86_64::structures::tss::TaskStateSegment;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const PANICKING_EXCEPTION_IST_INDEX: u16 = 1;
pub const IRQ_IST_INDEX: u16 = 2;

pub static TSS: Once<Mutex<RefCell<TaskStateSegment>>> = Once::new();

use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, DescriptorFlags as Flags,
                              SegmentSelector};


lazy_static! {
    static ref GDT: Gdt = {
        let mut gdt = GlobalDescriptorTable::new();
        let kernel_cs = gdt.add_entry(Descriptor::kernel_code_segment());
        let kernel_ds = gdt.add_entry(Descriptor::UserSegment(
            (Flags::USER_SEGMENT | Flags::PRESENT).bits() | (1 << 41),
        ));

        let tss = unsafe {
            gdt.add_entry(Descriptor::tss_segment(&*TSS.wait().unwrap().lock().as_ptr()))
        };

        let user_cs = gdt.add_entry(Descriptor::UserSegment(
            (Flags::USER_SEGMENT | Flags::PRESENT | Flags::EXECUTABLE | Flags::LONG_MODE).bits()
            | (3 << 45) // ring 3
        ));
        let user_ds = gdt.add_entry(Descriptor::UserSegment( //RW bit & ring3
            (Flags::USER_SEGMENT | Flags::PRESENT).bits() | (1 << 41) | (3 << 45),
        ));

        Gdt {
            table: gdt,
            selectors: Selectors { kernel_cs, kernel_ds, user_cs, user_ds, tss },
        }
    };
}

struct Gdt {
    table: GlobalDescriptorTable,
    selectors: Selectors,
}

struct Selectors {
    kernel_cs: SegmentSelector,
    kernel_ds: SegmentSelector,
    user_cs: SegmentSelector,
    user_ds: SegmentSelector,
    tss: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::segmentation::*;
    use x86_64::instructions::tables::load_tss;

    debug!("gdt: initialising rust gdt");

    GDT.table.load();

    unsafe {
        set_cs(GDT.selectors.kernel_cs);
        load_tss(GDT.selectors.tss);

        // Reload selector registers
        load_ss(GDT.selectors.kernel_ds);
        load_ds(GDT.selectors.kernel_ds);
        load_es(GDT.selectors.kernel_ds);
        load_fs(GDT.selectors.kernel_ds);
        load_gs(GDT.selectors.kernel_ds);
    }

    // TODO - do not hard code these
    trace!("kernel cs = 0x{:x}", GDT.selectors.kernel_cs.0);
    trace!("kernel ds = 0x{:x}", GDT.selectors.kernel_ds.0);
    trace!("user cs = 0x{:x}", GDT.selectors.user_cs.0);
    trace!("user ds = 0x{:x}", GDT.selectors.user_ds.0);

    debug!("gdt: initialised");
}
