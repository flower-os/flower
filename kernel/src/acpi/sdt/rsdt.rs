use core::{mem, convert::TryFrom};
use util::CChar;
use acpi::rsdp::{RsdpV1, RsdpV2};
use super::{SdtHeader, ValidationError};

const RSDT_HEADER: &'static [CChar; 4] = b"RSDT";
const XSDT_HEADER: &'static [CChar; 4] = b"XSDT";

/// The width of a table pointer found in the XSDT or RSDT. Implemented for u32 and u64
pub trait TablePtrWidth: Copy {
    fn into_usize(self) -> usize;
}

impl TablePtrWidth for u32 {
    fn into_usize(self) -> usize { self as usize }
}

impl TablePtrWidth for u64 {
    fn into_usize(self) -> usize { self as usize }
}

/// An iterator over table addresses, generic over RSDT/XSDT via [TablePtrWidth]
#[derive(Debug, Clone)]
pub struct TableAddresses<W: TablePtrWidth> {
    entries: usize,
    cur: usize,
    base_ptr: *const W,
}

impl<W: TablePtrWidth> TableAddresses<W> {
    /// Create a [TableAddresses] iterator from the number of entries and the base pointer
    fn from(entries: usize, base_ptr: usize) -> Self {
        TableAddresses {
            entries,
            cur: 0,
            base_ptr: base_ptr as *const W
        }
    }
}

impl<W: TablePtrWidth> Iterator for TableAddresses<W> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur < self.entries {
            let addr = unsafe { *self.base_ptr.offset(self.cur as isize) };
            self.cur += 1;
            Some(addr.into_usize())
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.entries - self.cur, Some(self.entries - self.cur))
    }
}

/// Validates an RSDT/XSDT based on its header and address
fn validate(header: SdtHeader, address: usize, is_extended: bool) -> Result<(), ValidationError> {

    let expected_signature = if is_extended {
        XSDT_HEADER
    } else {
        RSDT_HEADER
    };

    super::validate(header, address, expected_signature)
}

impl TryFrom<RsdpV1> for (SdtHeader, TableAddresses<u32>) {
    type Error = ValidationError;

    fn try_from(rsdp_v1: RsdpV1) -> Result<Self, ValidationError> {
        let header = unsafe { *(rsdp_v1.rsdt_address as *const SdtHeader) };
        validate(header, rsdp_v1.rsdt_address as usize, false)?;

        let sdt_addresses: TableAddresses<u32> = TableAddresses::from(
            (header.len as usize - mem::size_of::<SdtHeader>()) / 4,
            rsdp_v1.rsdt_address as usize + mem::size_of::<SdtHeader>()
        );

        Ok((header, sdt_addresses))
    }
}

impl TryFrom<RsdpV2> for (SdtHeader, TableAddresses<u64>) {
    type Error = ValidationError;

    fn try_from(rsdp_v2: RsdpV2) -> Result<Self, ValidationError> {
        let header = unsafe { *(rsdp_v2.xsdt_address as *const SdtHeader) };
        validate(header, rsdp_v2.xsdt_address as usize, true)?;

        let sdt_addresses: TableAddresses<u64> = TableAddresses::from(
            (header.len as usize - mem::size_of::<SdtHeader>()) / 8,
            rsdp_v2.xsdt_address as usize + mem::size_of::<SdtHeader>()
        );

        Ok((header, sdt_addresses))
    }
}