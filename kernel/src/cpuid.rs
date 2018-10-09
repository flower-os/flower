use core::arch::x86_64::__cpuid;

const CPUID_GET_FEATURES: u32 = 0x1;

bitflags! {
    pub struct Features: u64 {
        const FPU = 1 << 0;
        const VME = 1 << 1;
        const DE = 1 << 2;
        const PSE = 1 << 3;
        const TSC = 1 << 4;
        const MSR = 1 << 5;
        const PAE = 1 << 6;
        const MCE = 1 << 7;
        const CX8 = 1 << 8;
        const APIC = 1 << 9;
        const SEP = 1 << 11;
        const MTRR = 1 << 12;
        const PGE = 1 << 13;
        const MCA = 1 << 14;
        const CMOV = 1 << 15;
        const PAT = 1 << 16;
        const PSE36 = 1 << 17;
        const PSN = 1 << 18;
        const CLF = 1 << 19;
        const DTES = 1 << 21;
        const ACPI = 1 << 22;
        const MMX = 1 << 23;
        const FXSR = 1 << 24;
        const SSE = 1 << 25;
        const SSE2 = 1 << 26;
        const SS = 1 << 27;
        const HTT = 1 << 28;
        const TM1 = 1 << 29;
        const IA64 = 1 << 30;
        const PBE = 1 << 31;
        const SSE3 = 1 << 32;
        const PCLMUL = 1 << 33;
        const DTES64 = 1 << 34;
        const MONITOR = 1 << 35;
        const DS_CPL = 1 << 36;
        const VMX = 1 << 37;
        const SMX = 1 << 38;
        const EST = 1 << 39;
        const TM2 = 1 << 40;
        const SSSE3 = 1 << 41;
        const CID = 1 << 42;
        const FMA = 1 << 44;
        const CX16 = 1 << 45;
        const ETPRD = 1 << 46;
        const PDCM = 1 << 47;
        const PCIDE = 1 << 49;
        const DCA = 1 << 50;
        const SSE4_1 = 1 << 51;
        const SSE4_2 = 1 << 52;
        const x2APIC = 1 << 53;
        const MOVBE = 1 << 54;
        const POPCNT = 1 << 55;
        const AES = 1 << 57;
        const XSAVE = 1 << 58;
        const OSXSAVE = 1 << 59;
        const AVX = 1 << 60;
    }
}

/// Requests CPUID features and returns a set of flags
pub fn features() -> Features {
    let result = unsafe { __cpuid(CPUID_GET_FEATURES) };
    Features::from_bits_truncate(result.edx as u64 | (result.ecx as u64) << 32)
}
