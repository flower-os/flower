use acpi::{AcpiHandler, PhysicalMapping};
use core::{ptr::NonNull, alloc::{GlobalAlloc, Layout}};
use util;

pub fn acpi_init() {

}

struct FlowerAcpiHandler;

impl AcpiHandler for FlowerAcpiHandler {
    fn map_physical_region<T>(
        &mut self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<T> {
        let frames = util::round_up_divide(size as u64, 4096) as usize;
        let physical_begin = physical_address / 4096;
        let ptr = ::HEAP.alloc_specific(physical_begin, frames);

        PhysicalMapping {
           physical_start: physical_begin,
           virtual_start: NonNull::new(ptr as *mut T).expect("Ran out of heap space!"),
           region_length: frames * 4096,
           mapped_length: frames * 4096,
        }

    }

    fn unmap_physical_region<T>(&mut self, mut region: PhysicalMapping<T>) {
        unsafe {
            ::HEAP.dealloc(
                region.virtual_start.as_mut() as *mut T as *mut u8,
                Layout::from_size_align(region.mapped_length, 4096).unwrap(),
            );
        }
    }
}
