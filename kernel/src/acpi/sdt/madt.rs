use core::mem;
use util::CChar;
use super::{SdtHeader, ValidationError};

pub const MADT_HEADER: &'static [CChar; 4] = b"APIC";

/// A structure representing the MADT. This is not faithful to the actual representation
pub struct Madt {
    pub header: SdtHeader,
    pub local_apic_address: u32,
    pub legacy_pics: bool,
    pub entries: MadtEntries
}

impl Madt {
    pub unsafe fn from_address(address: usize) -> Result<Self, ValidationError> {
        let madt_begin = *(address as *const MadtBegin);
        let entries = MadtEntries::from_madt_address(address);

        let madt = Madt {
            header: madt_begin.header,
            local_apic_address: madt_begin.local_apic_address,
            legacy_pics: madt_begin.flags.contains(MadtFlags::LEGACY_PICS),
            entries,
        };

        super::validate(madt.header, address, MADT_HEADER).map(|_|madt)
    }
}

#[derive(Debug, Clone)]
pub struct MadtEntries {
    madt_address: usize,
    madt_len: usize,
    current_address: usize,
}

impl Iterator for MadtEntries {
    type Item = MadtEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_address <= self.madt_address + self.madt_len {
            let entry = unsafe { MadtEntry::from_address(self.current_address) };
            self.current_address += entry.header().len as usize;
            Some(entry)
        } else {
            None
        }
    }
}

impl MadtEntries {
    unsafe fn from_madt_address(madt_address: usize) -> Self {
        let header = *(madt_address as *const SdtHeader);
        MadtEntries {
            madt_address,
            madt_len: header.len as usize,
            current_address: madt_address + mem::size_of::<MadtBegin>(),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MadtEntry {
    LocalApic(LocalApicEntry),
    IoApic(IoApicEntry),
    InterruptSourceOverride(InterruptSourceOverrideEntry),
    NmiSource(NmiSourceEntry),
    LocalApicNmi(LocalApicNmiEntry),
    LocalApicAddressOverride(LocalApicAddressOverrideEntry),
    Unimplemented(MadtEntryHeader),
}


impl MadtEntry {
    pub fn entry_type(&self) -> MadtEntryType {
        use self::MadtEntry::*;
        match *self {
            LocalApic(_) => MadtEntryType::LocalApic,
            IoApic(_) => MadtEntryType::IoApic,
            InterruptSourceOverride(_) => MadtEntryType::InterruptSourceOverride,
            NmiSource(_) => MadtEntryType::NmiSource,
            LocalApicNmi(_) => MadtEntryType::LocalApicNmi,
            LocalApicAddressOverride(_) => MadtEntryType::LocalApicAddressOverride,
            Unimplemented(header) => header.entry_type,
        }
    }

    pub fn header(&self) -> MadtEntryHeader {
        use self::MadtEntry::*;
        match *self {
            LocalApic(e) => e.header,
            IoApic(e) => e.header,
            InterruptSourceOverride(e) => e.header,
            NmiSource(e) => e.header,
            LocalApicNmi(e) => e.header,
            LocalApicAddressOverride(e) => e.header,
            Unimplemented(h) => h,
        }
    }

    pub unsafe fn from_address(address: usize) -> Self {
        use self::MadtEntry::*;
        let header = *(address as *const MadtEntryHeader);

        match header.entry_type {
            MadtEntryType::LocalApic =>
                LocalApic(*(address as *const _)),
            MadtEntryType::IoApic =>
                IoApic(*(address as *const _)),
            MadtEntryType::InterruptSourceOverride =>
                InterruptSourceOverride(*(address as *const _)),
            MadtEntryType::NmiSource =>
                NmiSource(*(address as *const _)),
            MadtEntryType::LocalApicNmi =>
                LocalApicNmi(*(address as *const _)),
            MadtEntryType::LocalApicAddressOverride =>
                LocalApicAddressOverride(*(address as *const _)),
            _ => Unimplemented(header),
        }
    }
}

/// A structure representing the beginning of the MADT (i.e the fields)
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C, packed)]
struct MadtBegin {
    header: SdtHeader,
    local_apic_address: u32,
    flags: MadtFlags,
}

bitflags! {
    pub struct MadtFlags: u32 {
        /// Whether the system has two legacy PICs
        const LEGACY_PICS  = 1 << 0;
    }
}

/// A structure representing a MADT entry header
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C, packed)]
pub struct MadtEntryHeader {
    pub entry_type: MadtEntryType,
    pub len: u8,
}

/// The MADT entry type
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum MadtEntryType {
    /// [LocalApicEntry]
    LocalApic = 0,
    /// [IoApicEntry]
    IoApic = 1,
    /// [InterruptSourceOverrideEntry]
    InterruptSourceOverride = 2,
    /// [NmiSourceEntry]
    NmiSource = 3,
    /// [LocalApicNmiEntry]
    LocalApicNmi = 4,
    /// [LocalApicAddressOverrideEntry]
    LocalApicAddressOverride = 5,
    IoSapic = 6,
    LocalSapic = 7,
    PlatformInterruptSources = 8,
    LocalX2Apic = 9,
    LocalX2ApicNmi = 10,
    Gic = 11,
    Gicd = 12, // TODO these ^
}

/// A struct representing a local APIC entry
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C, packed)]
pub struct LocalApicEntry {
    header: MadtEntryHeader,
    /// The ID of the processor the local APIC is attached to
    pub processor_id: u8,
    /// The ID of the APIC
    pub apic_id: u8,
    pub flags: LocalApicFlags,
}

bitflags! {
    pub struct LocalApicFlags: u32 {
        /// Whether the Local Apic is enabled
        const ENABLED = 1 << 0;
    }
}

/// A struct representing an IO APIC entry
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C, packed)]
pub struct IoApicEntry {
    header: MadtEntryHeader,
    /// The IO APIC ID
    pub id: u8,
    _reserved: u8,
    /// The address of the IO Apic MMIO area
    pub address: u32,
    /// The interrupt number at which this IO APIC begins
    pub global_system_interrupt_base: u32,
}

/// A struct representing an Interrupt Source Override entry. This is used to map an ISA IRQ to an
/// IO APIC IRQ
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C, packed)]
pub struct InterruptSourceOverrideEntry {
    header: MadtEntryHeader,
    pub bus: u8,
    pub source: u8,
    pub global_system_interrupt: u32,
    pub flags: MpsIntiFlags,
}

/// The MPS INTI flags (as described by the ACPI specification, revision 5)
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C, packed)]
pub struct MpsIntiFlags(u16);

/// Enum representing of the APIC I/O signals, found in the [MpsIntiFlags]
#[repr(u8)]
pub enum MpsIntiPolarity {
    /// Conforms to bus specification
    Bus = 0b00,
    ActiveHigh = 0b01,
    ActiveLow = 0b11,
    /// Reserved by the ACPI specification, revision 5
    Reserved = 0b10,
}

#[repr(u8)]
pub enum MpsIntiTriggerMode {
    /// Conforms to bus specification
    Bus = 0b00,
    EdgeTriggered = 0b01,
    LevelTriggered = 0b11,
    /// Reserved by the ACPI specification, revision 5
    Reserved = 0b10,
}

impl MpsIntiFlags {
    /// Get the polarity of the APIC I/O signals
    pub fn polarity(self) -> MpsIntiPolarity {
        // Perfectly safe: all possible values are accounted for in enum
        unsafe { mem::transmute::<u8, MpsIntiPolarity>((self.0 & 0b11) as u8) }
    }

    pub fn trigger_mode(self) -> MpsIntiTriggerMode {
        // Perfectly safe: all possible values are accounted for in enum
        unsafe { mem::transmute::<u8, MpsIntiTriggerMode>((self.0 & 0b1100 >> 2) as u8) }
    }
}

/// A struct representing a Non-Maskable Interrupt Source entry.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C, packed)]
pub struct NmiSourceEntry {
    header: MadtEntryHeader,
    pub flags: MpsIntiFlags,
    /// The Global System Interrupt of this Non-Maskable Interrupt
    pub global_system_interrupt: u32,
}

/// A struct representing a Local APIC NMI entry
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C, packed)]
pub struct LocalApicNmiEntry {
    header: MadtEntryHeader,
    /// The ACPI processor ID. Value of `0xFF` signifies that this entry applies to all processors
    processor_id: u8,
    flags: MpsIntiFlags,
    /// The LINT number of this NMI
    local_int_number: u8,
}

/// A struct representing a Local APIC Address Override entry.
///
/// If available, this address *must* be used instead of the one in the [MADT] itself. This is
/// because that is only a 32bit address, while this one is a 64bit address.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C, packed)]
pub struct LocalApicAddressOverrideEntry {
    header: MadtEntryHeader,
    _reserved: u16,
    pub local_apic_address: u64,
}