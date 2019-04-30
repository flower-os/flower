//! Various utilities
use core::sync::atomic::{Ordering, AtomicU64};
use crate::drivers::pit;

lazy_static! {
    pub static ref RNG: Random = Random::new();
}

/// A macro to implement [FromDiscriminator] on an enum with explicit discriminators.
/// Doesn't support generics or comments, but does support attributes, etc
macro_rules! from_discriminator {
    {
        $(#[$attr:meta])*
        enum $name:ident {
            $($member:ident = $discriminator:expr),+
            $(,)* // Ugly, but works
        }
    } => {
        $(#[$attr])*
        enum $name {
            $($member = $discriminator),+
        }

        impl ::util::FromDiscriminator for $name {

            fn from_discriminator(discriminator: u64) -> Result<Self, ::util::UnknownDiscriminator> {
                match discriminator {
                    $($discriminator => Ok($name::$member)),+,
                    unknown => Err(::util::UnknownDiscriminator(unknown))
                }
            }
        }
    };

    {
        $(#[$attr:meta])*
        pub enum $name:ident {
            $($member:ident = $discriminator:expr),+
            $(,)* // Ugly, but works
        }
    } => {
        $(#[$attr])*
        pub enum $name {
            $($member = $discriminator),+
        }

        impl crate::util::FromDiscriminator for $name {
            fn from_discriminator(discriminator: u64) -> Result<Self, crate::util::UnknownDiscriminator> {
                match discriminator {
                    $($discriminator => Ok($name::$member)),+,
                    unknown => Err(crate::util::UnknownDiscriminator(unknown))
                }
            }
        }
    };
}

pub struct UnknownDiscriminator(pub u64);

pub trait FromDiscriminator: Sized {
    fn from_discriminator(discriminator: u64) -> Result<Self, UnknownDiscriminator>;
}

macro_rules! constant_unroll {
    (
        for $for_var:ident in [$($item:expr),*] {
            $iter_:ident = $iter:ident.$iter_fn:ident(move |$iter_var:ident| $block:block);
        }
    ) => {
        {
            $(
                let $iter = {
                    let $for_var = $item;
                    let $iter = $iter.$iter_fn(move |$iter_var| { $block });
                    $iter
                };
            )*

            $iter
        }
    }
}

/// Round up integer division
pub const fn round_up_divide(x: u64, y: u64) -> u64 {
    (x + y - 1) / y
}

/// Adapted from https://github.com/rust-lang-nursery/compiler-builtins/blob/master/src/mem.rs#L44
pub unsafe fn memset_volatile(s: *mut u8, c: u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        core::ptr::write_volatile(s.offset(i as isize), c);
        i += 1;
    }
    s
}

pub unsafe fn memset_volatile_64bit(s: *mut u64, c: u64, n: usize) -> *mut u64 {
    assert!(n & 0b111 == 0, "n must a be multiple of 8");
    assert!(s as usize & 0b111 == 0, "ptr must be aligned to 8 bytes");
    let mut i = 0;
    while i < (n >> 3) {
        core::ptr::write_volatile(s.offset(i as isize), c);
        i += 1;
    }
    s
}

pub unsafe fn cr3_write(val: ::x86_64::PhysAddr) {
    // Taken from https://docs.rs/x86_64/0.2.14/src/x86_64/registers/control.rs.html#116-120
    asm!("mov $0, %cr3" :: "r" (val.as_u64()) : "memory")
}

// Taken from https://docs.rs/x86_64/0.2.14/src/x86_64/registers/control.rs.html#100
pub fn cr3() -> u64 {
    let value: u64;
    unsafe {
        asm!("mov %cr3, $0" : "=r" (value));
    }

    value
}

pub struct Random {
    seed: AtomicU64,
}

impl Random {
    fn new() -> Random {
        let time = pit::time_ms() as u64;
        Random {
            seed: AtomicU64::new(time ^ 2246577883182828989),
        }
    }

    pub fn next_bounded(&self, bound: u64) -> u64 {
        self.next() % bound
    }

    /// Thanks to https://stackoverflow.com/a/3062783/4871468 and
    /// https://en.wikipedia.org/wiki/Linear_congruential_generator#Parameters_in_common_use
    /// (glibc's values used here).
    pub fn next(&self) -> u64 {
        const A: u64 = 1103515245;
        const M: u64 = 1 << 31;
        const C: u64 = 12345;

        let mut seed = self.seed.load(Ordering::SeqCst);
        loop {
            let next = (A.wrapping_mul(seed) + C) % M;
            let cas_result = self.seed.compare_and_swap(seed, next, Ordering::SeqCst);

            if cas_result == seed {
                return next;
            } else {
                seed = cas_result;
            }
        }
    }
}
