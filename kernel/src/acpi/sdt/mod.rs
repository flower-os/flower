use core::{num::Wrapping, fmt::{self, Display}};
use util::CChar;

pub mod rsdt;
pub mod madt;

/// Validate an SDT by checking if the signature is correct and checking if all bytes in the table
/// add to 0
fn validate(
    header: SdtHeader,
    address: usize,
    expected_signature: &'static [CChar; 4]
) -> Result<(), ValidationError> {
    // Check the signature is correct
    if header.signature != *expected_signature {
        return Err(ValidationError::UnexpectedSignature {
            expected: expected_signature,
            read: header.signature,
        });
    }

    // Checksum the MADT by adding all the bytes together and checking if equal to 0
    let base_address = address as *const u8;
    let sum = (0..header.len)
        .map(|offset| Wrapping(unsafe { *base_address.offset(offset as isize) }))
        .sum::<Wrapping<u8>>().0;

    if sum == 0 {
        Ok(())
    } else {
        Err(ValidationError::InvalidByteSum(sum))
    }
}

/// An enum representing an error validating an SDT
#[derive(Debug)]
pub enum ValidationError {
    /// The sum of all the bytes in the table was not zero
    InvalidByteSum(u8),
    /// The signature was unexpected
    UnexpectedSignature {
        expected: &'static [CChar; 4],
        read: [CChar; 4],
    }
}

/// The raw ACPI layout of the SDT header
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C, packed)]
pub struct SdtHeader {
    /// The signature of the table, e.g "RSDT"
    pub signature: [CChar; 4],
    /// The length of the table
    pub len: u32,
    /// The ACPI revision for this table
    pub revision: u8,
    _checksum: u8,
    /// The OEM ID, e.g "BOCHS"
    pub oem_id: [CChar; 6],
    /// The manufacturer model id (for the RSDT). Must match the same field in FADT
    pub oem_table_id: [CChar; 8],
    pub oem_revision: u32,
    /// ID of the tool that generated the table, e.g ID of ASL compiler
    pub creator_id: [CChar; 4],
    /// Revision of the tool that generated the table, e.g revision of ASL compiler
    pub creator_revision: u32,
}

impl Display for SdtHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let SdtHeader { len, revision, oem_revision, creator_revision, ..} = *self;

        write!(f, "Header\n------\n")?;
        write!(f, "Signature: \"")?;

        for c in self.signature.iter() {
            write!(f, "{}", *c as char)?;
        }

        write!(f, "\"\n")?;
        write!(f, "Length: {}\nRevision: {}\nOEM ID: \"", len, revision)?;

        for c in self.oem_id.iter() {
            write!(f, "{}", *c as char)?;
        }

        write!(f, "\"\n")?;
        write!(f, "OEM Table ID: \"")?;

        for c in self.oem_table_id.iter() {
            write!(f, "{}", *c as char)?;
        }

        write!(f, "\"\n")?;
        write!(f, "OEM Revision: {}\nCreator ID:\"", oem_revision)?;

        for c in self.creator_id.iter() {
            write!(f, "{}", *c as char)?;
        }

        write!(f, "\"\n")?;
        write!(f, "Creator Revision: {}", creator_revision)
    }
}