use core::alloc::Layout;
use multiboot2::BootInformation;
use memory::paging::{self, Page, PhysicalAddress, EntryFlags, page_map::TemporaryPage};
use memory::{bootstrap_heap::BOOTSTRAP_HEAP, physical_allocator::PHYSICAL_ALLOCATOR};
use memory::heap::Heap;
use memory::paging::PageSize;

pub fn remap_kernel(boot_info: &BootInformation, heap_tree_start: usize) {
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
    let heap_frame_addr = paging::PAGE_TABLES.lock().walk_page_table(heap_page).unwrap().0;

    // Unmap the heap page temporarily to avoid confusing the temporary page code
    // This code *is* correct -- we want to avoid mapping arbitrary pages as temporary, so we keep it in
    unsafe {
        paging::PAGE_TABLES.lock().unmap(heap_page, false, true);
        trace!("Unmapped page {:?}", heap_page);
    }

    let mut temporary_page = TemporaryPage::new(
        Page::containing_address(heap_page_addr as usize, PageSize::Kib4)
    );

    trace!("Creating new page tables");

    let mut active_table = unsafe { paging::ActivePageMap::new() };
    let mut new_table = {
        let frame = PhysicalAddress(
            PHYSICAL_ALLOCATOR.allocate(0).expect("no more frames") as usize
        );
        paging::InactivePageMap::new(frame, &mut active_table, &mut temporary_page)
    };

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

            if !section.flags().contains(ElfSectionFlags::ALLOCATED) {
                continue;
            }

            if section.flags().contains(ElfSectionFlags::WRITABLE) {
                flags = flags | paging::EntryFlags::WRITABLE;
            }

            if !section.flags().contains(ElfSectionFlags::EXECUTABLE) {
                flags = flags | paging::EntryFlags::NO_EXECUTE;
            }

            unsafe {
                mapper.higher_half_map_range(
                    section.start_address() as usize..=section.end_address() as usize,
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

            // Map bootstrap heap
            mapper.higher_half_map_range(
                BOOTSTRAP_HEAP.start()..=BOOTSTRAP_HEAP.end(),
                EntryFlags::WRITABLE | EntryFlags::NO_EXECUTE,
                false
            );
        }
    });

    trace!("Done mapping new pages");

    // Map heap pages
    let heap_tree_end = heap_tree_start + Heap::tree_size();
    for page_no in (heap_tree_start / 4096)..=(heap_tree_end / 4096) {
        let page = Page::containing_address(page_no * 4096, PageSize::Kib4);
        let phys_addr = paging::PAGE_TABLES.lock().walk_page_table(page).unwrap().0;

        active_table.with_inactive_p4(&mut new_table, &mut temporary_page, |mapper| {
            unsafe {
                mapper.map_to(
                    page,
                    phys_addr.physical_address().unwrap(),
                    EntryFlags::WRITABLE | EntryFlags::NO_EXECUTE,
                    false
                );
            }
        });
    }

    // Remap heap page so it can be deallocated correctly
    unsafe {
        paging::PAGE_TABLES.lock().map_to(
            heap_page,
            heap_frame_addr.physical_address().unwrap(),
            paging::EntryFlags::from_bits_truncate(0),
            true, // Invplg
        );
    }

    trace!("Deallocating heap page");
    unsafe { ::HEAP.dealloc(heap_page_addr, heap_layout) };

    trace!("mem: switching page tables");
    active_table.switch(new_table);

    trace!("mem: enabling write protection");
    unsafe { cr0_write(cr0() | Cr0::WRITE_PROTECT) };
}
