//! Various utilities

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