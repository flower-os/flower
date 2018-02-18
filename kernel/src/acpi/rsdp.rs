use core::{fmt::{self, Display}, ptr, mem};
use either::Either;
use util::CChar;

const EBDA_START_SEGMENT_PTR: usize = 0x40e;
const EBDA_EARLIEST_START: usize = 0x80000;
const EBDA_END: usize = 0x9ffff;
const RSDP_BIOS_AREA_START: usize = 0xe0000;
const RSDP_BIOS_AREA_END: usize = 0xfffff;
const RSDP_SIGNATURE: [CChar; 8] = cchar_string!('R', 'S', 'D', ' ', 'P', 'T', 'R', ' ');

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct SearchArea {
    begin: usize,
    end: usize,
}

fn find_ebda_start() -> usize {
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

/// Search for the rsdp, returning either RSDP from v1 acpi or RSDP from v2 acpi alone with
/// where it was found, else nothing.
pub fn search_for_rsdp() -> Option<(Either<RsdpV1, RsdpV2>, usize)> {
    let ebda_start = find_ebda_start();

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

        if signature != RSDP_SIGNATURE {
            continue;
        }

        let rsdp_v1: RsdpV1 = unsafe { ptr::read(address as *const _) };

        if !rsdp_v1.validate() {
            println!("acpi: found invalid v1 rsdp\n      at {:#x}", address);
                    //RSDT Address: {:#x}
            continue;
        }

        if rsdp_v1.revision > 1 {
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

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct RsdpV1 {
    pub signature: [CChar; 8],
    pub _checksum: u8,
    pub oem_id: [CChar; 6],
    pub revision: u8,
    pub rsdt_address: u32,
}

impl Display for RsdpV1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f,
               "RSDP Table
----------
")?;

        write!(f, "OEM ID: \"")?;

        for c in self.oem_id.iter() {
            write!(f, "{}", c)?;
        }

        // Copy from struct to avoid referencing to packed value
        let revision = self.revision;
        let rsdt_address = self.rsdt_address;

        write!(f,
               r#""
Revision: {}
RSDT Address: {:#x}"#, revision, rsdt_address)
    }
}

impl RsdpV1 {
    pub fn validate(&self) -> bool {
        let bytes: [u8; mem::size_of::<Self>()] = unsafe { mem::transmute_copy(&self) };

        // Make sure lowest byte is equal to 0
        bytes.into_iter().map(|i| *i as u64).sum::<u64>() >> 48 == 0
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct RsdpV2 {
    pub v1: RsdpV1,
    pub len: u32,
    pub xsdt_address: u32,
    pub _extended_checksum: u8,
    pub _reserved: [u8; 3],
}

impl RsdpV2 {
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

        write!(f,
               r#"
Length: {}
XSDT Adress: {:#x}"#, len, xsdt_address)
    }
}