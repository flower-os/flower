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
