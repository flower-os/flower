use core::{ops::Range, cell::RefCell};
use spin::{Once, Mutex};
use x86_64::structures::tss::TaskStateSegment;
use bit_field::BitField;
use crate::serial::PORT_1_ADDR;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const PANICKING_EXCEPTION_IST_INDEX: u16 = 1;
pub const IRQ_IST_INDEX: u16 = 2;

pub static TSS: Once<Mutex<Tss>> = Once::new();

use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, DescriptorFlags as Flags,
                              SegmentSelector};


lazy_static! {
    static ref GDT: Gdt = {
        let mut gdt = GlobalDescriptorTable::new();
        let kernel_cs = gdt.add_entry(Descriptor::kernel_code_segment());
        let kernel_ds = gdt.add_entry(Descriptor::UserSegment(
            (Flags::USER_SEGMENT | Flags::PRESENT).bits() | (1 << 41),
        ));

        let tss = unsafe { gdt.add_entry(
            Descriptor::tss_segment(&*TSS.wait().unwrap().lock().tss.as_ptr(), 8193)
        )};

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

#[repr(C)]
pub struct Tss {
    pub tss: RefCell<TaskStateSegment>,
    iomap: [u8; 8193],
}

impl Tss {
    pub fn new(tss: TaskStateSegment) -> Self {
        let mut tss = Tss {
            tss: RefCell::new(tss),
            iomap: [0xff; 8193],
        };

        // Absolute values don't matter, only the difference
        let tss_addr = tss.tss.as_ptr() as usize;
        let iomap_addr = (&tss.iomap) as *const _ as usize;
        let iomap_base = (iomap_addr - tss_addr) as u16;

        tss.tss.get_mut().iomap_base = iomap_base;

        tss
    }

    pub fn set_ports_usable(&mut self, ports: Range<u16>, usable: bool) {
        assert!(ports.end / 8 < 8192, "Port 0x{:x} out of bounds", ports.end);

        // TODO could be optimised
        for port in ports {
            let byte_idx = port / 8;
            let bit = port % 8;
            // For some reason 1 = disabled
            self.iomap[byte_idx as usize].set_bit(bit as usize, !usable);
        }
    }

    pub fn is_port_usable(&self, port: u16) -> bool {
        assert!(port / 8 < 8192, "Port 0x{:x} out of bounds", port);

        let byte_idx = port / 8;
        let bit = port % 8;
        // For some reason 1 = disabled
        !self.iomap[byte_idx as usize].get_bit(bit as usize)
    }
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


    TSS.wait().unwrap().lock().set_ports_usable(PORT_1_ADDR..PORT_1_ADDR + 8, true);
    trace!("port enabled: {:?}", TSS.wait().unwrap().lock().is_port_usable(PORT_1_ADDR));
    debug!("gdt: initialised");
}
