use core::{mem, ptr, alloc::Layout};
use multiboot2::BootInformation;
use memory::paging::{self, PAGE_TABLES, Page, PhysicalAddress, EntryFlags, page_map::TemporaryPage};
use memory::{bootstrap_heap::BOOTSTRAP_HEAP, physical_allocator::PHYSICAL_ALLOCATOR};
use memory::heap::{self, Heap};
use memory::paging::PageSize;
use util;
use memory::paging::page_map::PageRangeMapping;
use alloc::vec::Vec;

pub fn remap_kernel(
    boot_info: &BootInformation,
    heap_tree_start_virt: usize,
//    heap_tree_start_phys: usize // Could be done better TODO
) {
    use core::alloc::GlobalAlloc;
    use multiboot2::ElfSectionFlags;
    use x86_64::registers::control_regs::{cr0, cr0_write, Cr0};

    // Allocate some heap memory for us to put the temporary page on
    let heap_layout = Layout::from_size_align(0x1000, 0x1000).unwrap();
    let heap_page_addr = unsafe {
        ::HEAP.alloc(heap_layout)
    };

    let heap_page = Page::containing_address(
        heap_page_addr as usize,
        PageSize::Kib4
    );
    let heap_frame_addr = PAGE_TABLES.lock().walk_page_table(heap_page).unwrap().0;

    // Unmap the heap page temporarily to avoid confusing the temporary page code
    // This code *is* correct -- we want to avoid mapping arbitrary pages as temporary, so we keep it in
    unsafe {
        PAGE_TABLES.lock().unmap(heap_page, false, true);
    }

    let mut temporary_page = TemporaryPage::new(heap_page);

    trace!("Creating new page tables");

    let mut active_table = unsafe { paging::ActivePageMap::new() };

    let frame = PhysicalAddress(
        PHYSICAL_ALLOCATOR.allocate(0).expect("no more frames") as usize
    );

    let paddr = heap_frame_addr.physical_address().unwrap().0 as *const u8;
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

            let mut flags = paging::EntryFlags::from_bits_truncate(0);

            if section.flags().contains(ElfSectionFlags::WRITABLE) {
                flags = flags | paging::EntryFlags::WRITABLE;
            }

            if !section.flags().contains(ElfSectionFlags::EXECUTABLE) {
                flags = flags | paging::EntryFlags::NO_EXECUTE;
            }

            unsafe {
                mapper.higher_half_map_range(
                    section.start_address() as usize..section.end_address() as usize,
                    flags,
                    false
                );
            }
        }

        unsafe {
            // Map VGA buffer
            mapper.map_to(
                Page::containing_address(::drivers::vga::VIRTUAL_VGA_PTR, PageSize::Kib4),
                PhysicalAddress(0xb8000 as usize),
                EntryFlags::WRITABLE | EntryFlags::NO_EXECUTE,
                false,
            );
        }
    });

    // TODO
    debug!("bootstrap heap = 0x{:x}..0x{:x}", BOOTSTRAP_HEAP.start(), BOOTSTRAP_HEAP.end());

    // Map bootstrap heap
    let bootstrap_heap_start_page = BOOTSTRAP_HEAP.start() / 4096;
    let bootstrap_heap_end_page = util::round_up_divide(
        BOOTSTRAP_HEAP.end() as u64,
        4096
    ) as usize;
    let bootstrap_heap_page_range = bootstrap_heap_start_page..=bootstrap_heap_end_page;

    trace!("trace -1");

    active_table.remap_range(
        &mut new_table,
        &mut temporary_page,
        bootstrap_heap_page_range,
        paging::EntryFlags::NO_EXECUTE | paging::EntryFlags::WRITABLE
    );

    trace!("trace 0");

    // Map heap
    let heap_tree_start_page = heap_tree_start_virt / 4096;
    let heap_tree_end_page = util::round_up_divide(
        (heap_tree_start_virt + Heap::tree_size()) as u64,
        4096
    ) as usize;
    let heap_tree_page_range = heap_tree_start_page..=heap_tree_end_page;

    trace!("trace 1");
    active_table.remap_range(
        &mut new_table,
        &mut temporary_page,
        heap_tree_page_range,
        paging::EntryFlags::NO_EXECUTE | paging::EntryFlags::WRITABLE
    );
    trace!("trace 2");

    PHYSICAL_ALLOCATOR.is_free(paddr, 0);
    let addr = PAGE_TABLES.lock()
        .walk_page_table(Page::containing_address(0xffffffff803c3781, PageSize::Kib4))
        .unwrap().0.physical_address().unwrap().0;
    debug!("backing addr = 0x{:x}", addr);
    // TODO

    trace!("mem: switching page tables");
    active_table.switch(new_table);

    // Remap heap page so it can be deallocated correctly
    trace!("{:?}", heap_frame_addr.physical_address().unwrap());
    unsafe {
        PAGE_TABLES.lock().map_to(
            heap_page,
            heap_frame_addr.physical_address().unwrap(),
            paging::EntryFlags::from_bits_truncate(0),
            true, // Invplg
        );
    }

    let addr = PAGE_TABLES.lock()
        .walk_page_table(heap_page)
        .unwrap().0.physical_address().unwrap().0;
    debug!("virtual addr = 0x{:x}, backing addr = 0x{:x}", heap_page.start_address().unwrap(), addr);
    debug!("heap page addr = {:?}", heap_page_addr);
    unsafe { ::HEAP.dealloc(heap_page_addr, heap_layout) };

    trace!("dealloc'd heap page");

    trace!("mem: enabling write protection");
    unsafe { cr0_write(cr0() | Cr0::WRITE_PROTECT) };
}
