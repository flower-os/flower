use core::fmt::{self, Display};

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
            type Output = $name;

            fn from_discriminator(discriminator: u64) -> Result<Self::Output, ::util::UnknownDiscriminator> {
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
            type Output = $name;

            fn from_discriminator(discriminator: u64) -> Result<Self::Output, ::util::UnknownDiscriminator> {
                match discriminator {
                    $($discriminator => Ok($name::$member)),+,
                    unknown => Err(::util::UnknownDiscriminator(unknown))
                }
            }
        }
    };
}

pub struct UnknownDiscriminator(pub u64);

pub trait FromDiscriminator {
    type Output;
    fn from_discriminator(discriminator: u64) -> Result<Self::Output, UnknownDiscriminator>;
}

/// A struct representing a C `char`
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub struct CChar(pub i8);

impl CChar {
    /// Casts the char to an i8 and returns a wrapping CChar
    pub const fn from_char(c: char) -> Self {
        CChar(c as i8)
    }
}

impl Display for CChar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0 as u8 as char)
    }
}

/// Macro for creating a [CChar] string (array of [CChar]) without a null terminator
macro_rules! cchar_string {
    [$($c:expr),*] => (
        [$(::util::CChar::from_char($c)),*]
    )
}
