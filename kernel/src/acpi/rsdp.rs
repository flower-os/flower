//! RSDP (Root System Description Pointer) module

use core::{fmt::{self, Display}, ptr, mem};
use either::Either;
use util::CChar;

/// The pointer to the EBDA (Extended Bios Data Area) start segment pointer
const EBDA_START_SEGMENT_PTR: usize = 0x40e;
/// The earliest (lowest) memory address an EBDA (Extended Bios Data Area) can start
const EBDA_EARLIEST_START: usize = 0x80000;
/// The end of the EBDA (Extended Bios Data Area)
const EBDA_END: usize = 0x9ffff;
/// The start of the main bios area below 1mb in which to search for the RSDP
/// (Root System Description Pointer)
const RSDP_BIOS_AREA_START: usize = 0xe0000;
/// The end of the main bios area below 1mb in which to search for the RSDP
/// (Root System Description Pointer)
const RSDP_BIOS_AREA_END: usize = 0xfffff;
/// The RSDP (Root System Description Pointer)'s signature, "RSD PTR " (note trailing space)
const RSDP_SIGNATURE: &'static [CChar; 8] = b"RSD PTR ";

/// A structure describing an area of memory to search
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct SearchArea {
    pub begin: usize,
    pub end: usize,
}

/// Find the begining of the EBDA (Extended Bios Data Area)
fn find_ebda_start() -> usize {
    // Read base from BIOS area. This is not always given by the bios, so it needs to be checked
    let base = (unsafe { ptr::read(EBDA_START_SEGMENT_PTR as *const u16) } as usize) << 4;

    // Check if base segment ptr is in valid range valid
    if (EBDA_EARLIEST_START..EBDA_END).contains(base) {
        println!("acpi: EBDA address is {:#x}", base);
        base
    } else {
        println!(
            "acpi: EBDA address at {:#x} out of range\n      ({:#x}), falling back to {:#x}",
            EBDA_START_SEGMENT_PTR,
            base,
            EBDA_EARLIEST_START
        );

        EBDA_EARLIEST_START
    }
}

/// Search for the RSDP, returning either the RSDP from v1 acpi or the RSDP from v2 acpi along
/// with where it was found. If it could not be found, then `None` is returned.
pub fn search_for_rsdp() -> Option<(Either<RsdpV1, RsdpV2>, usize)> {
    let ebda_start = find_ebda_start();

    // The areas that will be searched for the RSDP
    let areas = [
        // Main bios area below 1 mb
        // In practice RSDP is more often here than in ebda
        SearchArea {
            begin: RSDP_BIOS_AREA_START,
            end: RSDP_BIOS_AREA_END,
        },

        // First kb of ebda
        SearchArea {
            begin: ebda_start,
            end: ebda_start + 1024,
        },
    ];

    let mut rsdp_and_addr: Option<(Either<RsdpV1, RsdpV2>, usize)> = None;

    // Signature is always on a 16 byte boundary so only search there
    for address in areas.iter().flat_map(|area| area.begin..area.end).step_by(16) {
        let signature: [CChar; 8] = unsafe { ptr::read(address as *const _) };

        if signature != *RSDP_SIGNATURE {
            continue;
        }

        let rsdp_v1: RsdpV1 = unsafe { ptr::read(address as *const _) };

        if !rsdp_v1.validate() {
            println!("acpi: found invalid v1 rsdp\n      at {:#x}", address);
            continue;
        }

        // If revision is > 0 (0 = ACPI v1) then use v2 and above RSDP
        if rsdp_v1.revision > 0 {
            let rsdp_v2: RsdpV2 = unsafe { ptr::read(address as *const _) };

            if !rsdp_v2.validate() {
                println!("acpi: found invalid v2 rsdp\n      at {:#x}", address);
                continue;
            }

            rsdp_and_addr = Some((Either::Right(rsdp_v2), address));
            break;
        } else {
            rsdp_and_addr = Some((Either::Left(rsdp_v1), address));
            break;
        }
    }

    rsdp_and_addr
}

/// The root system description pointer in ACPI v1. This is different to the RSDP in ACPI v2 and
/// above in that newer versions also point to the XSDT (eXtended System Description Table)
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct RsdpV1 {
    signature: [CChar; 8],
    _checksum: u8,
    pub oem_id: [CChar; 6],
    /// The ACPI revision. ACPI v1 revision is 0 for RSDP
    pub revision: u8,
    pub rsdt_address: u32,
}

impl Display for RsdpV1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "RSDP Table\n----------")?;
        write!(f, "OEM ID: \"")?;

        for c in self.oem_id.iter() {
            write!(f, "{}", *c as char)?;
        }

        write!(f, "\"")?;

        // Copy from struct to avoid referencing to packed value
        let revision = self.revision;
        let rsdt_address = self.rsdt_address;

        write!(f, "Revision: {}\nRSDT Address: {:#x}", revision, rsdt_address)
    }
}

impl RsdpV1 {
    /// Validates the RSDP by summing all bytes (first bit interpreted as sign) together and
    /// checking that the lowest byte is 0
    pub fn validate(&self) -> bool {
        let bytes: [u8; mem::size_of::<Self>()] = unsafe { mem::transmute_copy(&self) };

        // Make sure lowest byte is equal to 0
        bytes.into_iter().map(|i| *i as u64).sum::<u64>() >> 48 == 0
    }
}

/// The root system description pointer in ACPI v1. This is different to the RSDP in ACPI v1 in that
/// newer versions also point to the XSDT (eXtended System Description Table)
#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct RsdpV2 {
    pub v1: RsdpV1,
    pub len: u32,
    pub xsdt_address: u64,
    pub _extended_checksum: u8,
    pub _reserved: [u8; 3],
}

impl RsdpV2 {
    /// Validates the RSDP by summing all bytes (first bit interpreted as sign) together and
    /// checking that the lowest byte is 0
    pub fn validate(&self) -> bool {
        let bytes: [u8; mem::size_of::<Self>()] = unsafe { mem::transmute_copy(&self) };

        // Make sure lowest byte is equal to 0
        bytes.into_iter().map(|i| *i as u64).sum::<u64>() >> 48 == 0
    }
}

impl Display for RsdpV2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.v1.fmt(f)?;

        // Copy from struct to avoid referencing to packed value
        let len = self.len;
        let xsdt_address = self.xsdt_address;

        write!(f, "Length: {}\nXSDT Adress: {:#x}", len, xsdt_address)
    }
}