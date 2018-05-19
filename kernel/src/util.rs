//! Various utilities

use core::{u32, f32};

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

        impl ::util::FromDiscriminator for $name {
            fn from_discriminator(discriminator: u64) -> Result<Self, ::util::UnknownDiscriminator> {
                match discriminator {
                    $($discriminator => Ok($name::$member)),+,
                    unknown => Err(::util::UnknownDiscriminator(unknown))
                }
            }
        }
    };
}

pub struct UnknownDiscriminator(pub u64);

pub trait FromDiscriminator: Sized {
    fn from_discriminator(discriminator: u64) -> Result<Self, UnknownDiscriminator>;
}

/// Calculate the ceil of a _positive_ f32.
pub fn ceil(num: f32) -> u32 {
    assert!(num >= 0.0 && num < (u32::MAX as f32), "Tried to take ceil of {}", num);

    if num - (num as u32 as f32) < f32::EPSILON {
        num as u32
    } else {
        num as u32 + 1
    }
}