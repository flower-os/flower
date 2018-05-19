//! # HPET: High Precision Event Timer
//!
//! This module handles parsing of the HPET table
//! From [HPET Spec](https://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/software-developers-hpet-spec-1-0a.pdf)

use super::{CChar, SdtHeader, ValidationError};

/// Expected header from the HPET table
pub const HPET_HEADER: &'static [CChar; 4] = b"HPET";

/// Represents a parsed HPET table
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Hpet {
    /// Header for the HPET table
    pub header: SdtHeader,
    /// Hardware revision id
    pub hardware_rev_id: u8,
    /// Amount of comparators registered to 1st timer block
    pub comparator_count: u8,
    /// Size in bits of the HPET counter
    pub counter_size: CounterSize,
    /// Whether this is LegacyReplacement IRQ Routing Capable
    pub legacy_irq_routing: bool,
    /// The PCI vendor ID of 1st timer block
    pub pci_vendor_id: u16,
    /// The minimum clock ticks can be set without lost interrupts while the counter is programmed
    /// to operate in periodic mode
    pub clock_tick_unit: u16,
}

impl Hpet {
    pub unsafe fn from_address(address: usize) -> Result<Self, ValidationError> {
        let hpet_begin = *(address as *const HpetBegin);
        super::validate(hpet_begin.header, address, HPET_HEADER)?;

        let hardware_rev_id = (hpet_begin.event_timer_block_id & 255) as u8;
        let comparator_count = ((hpet_begin.event_timer_block_id >> 8) & 31) as u8;
        let counter_size = if ((hpet_begin.event_timer_block_id >> 13) & 1) != 0 {
            CounterSize::Size64
        } else {
            CounterSize::Size32
        };
        let legacy_irq_routing = ((hpet_begin.event_timer_block_id >> 15) & 1) != 0;
        let pci_vendor_id = ((hpet_begin.event_timer_block_id >> 16) & 65535) as u16;

        Ok(Hpet {
            header: hpet_begin.header,
            hardware_rev_id,
            comparator_count,
            counter_size,
            legacy_irq_routing,
            pci_vendor_id,
            clock_tick_unit: hpet_begin.clock_tick_unit,
        })
    }
}

/// Size of the timer counter
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CounterSize {
    Size32,
    Size64,
}

/// Struct representing the packed HPET table
#[repr(C, packed)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct HpetBegin {
    header: SdtHeader,
    event_timer_block_id: u32,
    base_address: BaseAddress,
    hpet_number: u8,
    clock_tick_unit: u16,
    page_protection_oem: u8,
}

/// The lower 32-bit base address of Event Timer Block. Each Event Timer Block consumes 1K
/// of system memory, regardless of how many comparators are actually implemented by the hardware.
#[repr(C, packed)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct BaseAddress {
    address_space_id: u8,
    register_bit_width: u8,
    regsiter_bit_offset: u8,
    _reserved: u8,
    address: u64,
}
