use acpi::{self, AcpiHandler, PhysicalMapping};
use core::{ptr::NonNull, alloc::{GlobalAlloc, Layout}};
use util;

pub fn acpi_init() {
    info!("acpi: initialising");
    let mut handler = FlowerAcpiHandler;
    // We're BIOS. We'd have crashed by now if we weren't.
    unsafe { acpi::search_for_rsdp_bios(&mut handler).expect("ACPI error") };
    info!("apci: initialised");
}


struct FlowerAcpiHandler;

impl AcpiHandler for FlowerAcpiHandler {
    fn map_physical_region<T>(
        &mut self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<T> {
        let frames = util::round_up_divide(size as u64, 4096) as usize;
        let physical_begin_frame = physical_address / 4096;

        let alloc_ptr = ::HEAP.alloc_specific(physical_begin_frame, frames) as usize;
        let obj_ptr = alloc_ptr + physical_address - (physical_begin_frame * 4096);

        let p = PhysicalMapping {
           physical_start: physical_begin_frame * 4096,
           // alloc_ptr is zero if there is no more heap memory available
           virtual_start: NonNull::new(obj_ptr as *mut T)
               .expect("Ran out of heap memory!"),
           region_length: frames * 4096,
           mapped_length: frames * 4096,
        };
        trace!("Mapping {:?}", p.virtual_start);
        p
    }

    fn unmap_physical_region<T>(&mut self, mut region: PhysicalMapping<T>) {
        trace!("Unmapping {:?}", region.virtual_start);
        let obj_addr = region.virtual_start.as_ptr() as *mut T as usize;

        // Clear lower page offset bits
        let page_begin = obj_addr & !0xFFF;

        unsafe {
            ::HEAP.dealloc(
                page_begin as *mut u8,
                Layout::from_size_align(region.mapped_length, 4096).unwrap(),
            );
        }
    }
}
