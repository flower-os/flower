use acpi::{self, AcpiHandler, Acpi, AcpiError};
use core::ptr::NonNull;
use memory::{self, PhysicalMapping};
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
        }
        Err(e) => {
            error!("acpi: init unsuccessful {:?}", e);
            Err(e)
        }
    }
}

struct FlowerAcpiHandler;

impl AcpiHandler for FlowerAcpiHandler {
    fn map_physical_region<T>(
        &mut self,
        physical_address: usize,
        size: usize,
    ) -> acpi::PhysicalMapping<T> {
        let region: PhysicalMapping<T> = memory::map_physical_region(physical_address, size);
        region.into()
    }

    fn unmap_physical_region<T>(&mut self, region: acpi::PhysicalMapping<T>) {
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
