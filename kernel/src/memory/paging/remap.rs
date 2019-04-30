use x86_64::registers::control::{Cr0, Cr0Flags};
use multiboot2::{BootInformation, ElfSectionFlags};
use crate::memory::paging::{self, ACTIVE_PAGE_TABLES, Page, PhysicalAddress, EntryFlags,
                            page_map::TemporaryPage, InvalidateTlb, FreeMemory};
use crate::memory::{bootstrap_heap::BOOTSTRAP_HEAP, physical_allocator::PHYSICAL_ALLOCATOR};
use crate::memory::heap::Heap;
use crate::memory::paging::PageSize;
use crate::util;

pub fn remap_kernel(
    boot_info: &BootInformation,
    heap_tree_start_virt: usize,
) {
    let mut temporary_page = TemporaryPage::new();

    trace!("Creating new page tables");

    // This must be duplicated to avoid double locks. This is safe though -- in this context!
    let mut active_table = unsafe { paging::ActivePageMap::new() };
    let frame = PhysicalAddress(PHYSICAL_ALLOCATOR.allocate(0).expect("no more frames") as usize);
    let mut new_table = paging::InactivePageMap::new(frame, &mut active_table, &mut temporary_page);

    trace!("Mapping new page tables");

    active_table.with_inactive_p4(&mut new_table, &mut temporary_page, |mapper| {
        let elf_sections_tag = boot_info.elf_sections_tag()
            .expect("Memory map tag required");

        // Map kernel sections
        for section in elf_sections_tag.sections() {
            if !section.is_allocated() { continue; }

            assert_eq!(
                section.start_address() % 4096,
                0,
                "Section {} needs to be page aligned!",
                section.name(),
            );

            let mut flags = EntryFlags::from_bits_truncate(0) | EntryFlags::USER_ACCESSIBLE; // TODO

            if section.flags().contains(ElfSectionFlags::WRITABLE) {
                flags = flags | EntryFlags::WRITABLE;
            }

            if !section.flags().contains(ElfSectionFlags::EXECUTABLE) {
                flags = flags | EntryFlags::NO_EXECUTE;
            }

            unsafe {
                mapper.higher_half_map_range(
                    section.start_address() as usize..section.end_address() as usize,
                    flags,
                    InvalidateTlb::NoInvalidate,
                );
            }
        }

        unsafe {
            // Map VGA buffer
            mapper.map_to(
                Page::containing_address(crate::drivers::vga::VIRTUAL_VGA_PTR, PageSize::Kib4),
                PhysicalAddress(0xb8000 as usize),
                EntryFlags::WRITABLE | EntryFlags::NO_EXECUTE | EntryFlags::USER_ACCESSIBLE, // TODO
                InvalidateTlb::NoInvalidate,
            );
        }
    });

    // Map bootstrap heap
    let bootstrap_heap_start_page = BOOTSTRAP_HEAP.start() / 4096;
    let bootstrap_heap_end_page = util::round_up_divide(
        BOOTSTRAP_HEAP.end() as u64,
        4096
    ) as usize;
    let bootstrap_heap_page_range = bootstrap_heap_start_page..=bootstrap_heap_end_page;

    active_table.remap_range(
        &mut new_table,
        &mut temporary_page,
        bootstrap_heap_page_range,
        EntryFlags::NO_EXECUTE | EntryFlags::WRITABLE | EntryFlags::USER_ACCESSIBLE // TODO
    );

    // Map heap
    let heap_tree_start_page = heap_tree_start_virt / 4096;
    let heap_tree_end_page = util::round_up_divide(
        (heap_tree_start_virt + Heap::tree_size()) as u64,
        4096
    ) as usize;
    let heap_tree_page_range = heap_tree_start_page..=heap_tree_end_page;

    active_table.remap_range(
        &mut new_table,
        &mut temporary_page,
        heap_tree_page_range,
        EntryFlags::NO_EXECUTE | EntryFlags::WRITABLE | EntryFlags::USER_ACCESSIBLE // TODO
    );

    trace!("mem: switching page tables");
    active_table.switch(&new_table);

    // Drop this lock so that the RAII guarded temporary page can be destroyed
    drop(active_table);

    trace!("mem: enabling write protection");
    unsafe { Cr0::write(Cr0::read() | Cr0Flags::WRITE_PROTECT) };
}
