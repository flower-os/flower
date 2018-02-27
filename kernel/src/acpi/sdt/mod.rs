use core::fmt::{self, Display};
use util::CChar;

pub mod rsdt;

/// The raw ACPI layout of the SDT header
#[derive(Copy, Clone, Debug)]
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