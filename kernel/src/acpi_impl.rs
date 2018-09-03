use acpi::{self, AcpiHandler, PhysicalMapping, Acpi, AcpiError};
use core::ptr::NonNull;
use util;

pub fn acpi_init() -> Result<Acpi, AcpiError> {
    info!("acpi: initializing");
    let mut handler = FlowerAcpiHandler;
    // We're BIOS. We'd have crashed by now if we weren't.
    let search_result = unsafe { acpi::search_for_rsdp_bios(&mut handler) };
    match search_result {
        Ok(acpi) => {
            info!("acpi: init successful");
            Ok(acpi)
        },
        Err(e) => {
            error!("acpi: init unsuccessful {:?}", e);
            Err(e)
        },
    }
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

        let alloc_ptr = unsafe {
            ::HEAP.alloc_specific(physical_begin_frame, frames) as usize
        };

        if alloc_ptr == 0 {
            panic!("Ran out of heap memory!");
        }

        let obj_ptr = alloc_ptr + physical_address - (physical_begin_frame * 4096);

        PhysicalMapping {
           physical_start: physical_begin_frame * 4096,
           // alloc_ptr is zero if there is no more heap memory available
           virtual_start: NonNull::new(obj_ptr as *mut T)
               .expect("Ran out of heap memory!"),
           region_length: frames * 4096,
           mapped_length: frames * 4096,
        }
    }

    fn unmap_physical_region<T>(&mut self, region: PhysicalMapping<T>) {
        let obj_addr = region.virtual_start.as_ptr() as *mut T as usize;

        // Clear lower page offset bits
        let page_begin = obj_addr & !0xFFF;

        unsafe {
            ::HEAP.dealloc_specific(
                page_begin as *mut u8,
                region.mapped_length / 4096,
            );
        }
    }
}
