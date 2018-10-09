use core::arch::x86_64::__cpuid;

const CPUID_GET_FEATURES: u32 = 0x1;

bitflags! {
    /// Represents the collection of flags returned from `CPUID_GET_FEATURES`. This is combined into
    /// a `u64` from the two `u32`s returned by the CPUID command.
    ///
    /// Documentation of these opcodes can be found in:
    ///  - (AMD CPUID Spec)[https://www.amd.com/system/files/TechDocs/25481.pdf]
    ///  - (Intel CPUID Spec)[http://bochs.sourceforge.net/techspec/24161821.pdf]
    pub struct Features: u64 {
        const FLOATING_POINT_UNIT = 1 << 0;
        const VIRTUAL_MODE_EXTENSION = 1 << 1;
        const DEBUGGING_EXTENSION = 1 << 2;
        const PAGE_SIZE_EXTENSION = 1 << 3;
        const TIME_STAMP_COUNTER = 1 << 4;
        const MODEL_SPECIFIC_REGISTERS = 1 << 5;
        const PHYSICAL_ADDRESS_EXTENSION = 1 << 6;
        const MACHINE_CHECK_EXCEPTION = 1 << 7;
        const CMPXCHG8_INSTRUCTION = 1 << 8;
        const APIC = 1 << 9;
        const FAST_SYSTEM_CALL = 1 << 11;
        const MEMORY_TYPE_RANGE_REGISTERS = 1 << 12;
        const PAGE_GLOBAL_ENABLE = 1 << 13;
        const MACHINE_CHECK_ARCHITECTURE = 1 << 14;
        const CMOV_INSTRUCTION = 1 << 15;
        const PAGE_ATTRIBUTE_TABLE = 1 << 16;
        const PAGE_SIZE_36_EXTENSION = 1 << 17;
        const PROCESSOR_SERIAL_NUMBER_PRESENT = 1 << 18;
        const CLFLUSH_INSTRUCTION = 1 << 19;
        const DEBUG_STORE = 1 << 21;
        const ACPI = 1 << 22;
        const INTEL_MMX = 1 << 23;
        const FAST_FLOATING_POINT_SAVE_RESTORE = 1 << 24;
        const STREAMING_SIMD_EXTENSIONS = 1 << 25;
        const STREAMING_SIMD_EXTENSIONS_2 = 1 << 26;
        const SELF_SNOOP = 1 << 27;
        const HYPER_THREADING_TECHNOLOGY = 1 << 28;
        const THERMAL_MONITOR = 1 << 29;
        const STREAMING_SIMD_EXTENSIONS_3 = 1 << 32;
        const PCLMUL_INSTRUCTION = 1 << 33;
        const MONITOR_INSTRUCTION = 1 << 35;
        const SUPPLEMENTAL_STREAMING_SIMD_EXTENSIONS_3 = 1 << 41;
        const FMA_INSTRUCTION = 1 << 44;
        const CMPXCHG16B = 1 << 45;
        const STREAMING_SIMD_EXTENSION_4_1 = 1 << 51;
        const STREAMING_SIMD_EXTENSION_4_2 = 1 << 52;
        const x2APIC = 1 << 53;
        const POPCNT_INSTRUCTION = 1 << 55;
        const AES_INSTRUCTION = 1 << 57;
        const XSAVE_INSTRUCTION = 1 << 58;
        const OSXSAVE_INSTRUCTION = 1 << 59;
        const AVX_INSTRUCTION = 1 << 60;
        const F16C_INSTRUCTION = 1 << 61;
    }
}

/// Requests CPUID features and returns a set of flags
pub fn features() -> Features {
    let result = unsafe { __cpuid(CPUID_GET_FEATURES) };
    Features::from_bits_truncate(result.edx as u64 | (result.ecx as u64) << 32)
}
