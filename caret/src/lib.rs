//! `caret`: Integers with some named values.
//!
//! # Crikey! Another Rust Enum Tool?
//!
//! Suppose you have an integer type with some named values.  For
//! example, you might be implementing a protocol where "command" can
//! be any 8-bit value, but where only a small number of commands are
//! recognized.
//!
//! In that case, you can use the [`caret_int`] macro to define a
//! wrapper around `u8` so named values are displayed with their
//! preferred format, but you can still represent all the other values
//! of the field:
//!
//! ```
//! use caret::caret_int;
//! caret_int!{
//!     struct Command(u8) {
//!        Get = 0,
//!        Put = 1,
//!        Swap = 2,
//!     }
//! }
//!
//! let c1: Command = 2.into();
//! let c2: Command = 100.into();
//!
//! assert_eq!(c1.to_string().as_str(), "Swap");
//! assert_eq!(c2.to_string().as_str(), "100");
//!
//! assert_eq!(c1, Command::Swap);
//! ```
//!
//! This crate is developed as part of
//! [Arti](https://gitlab.torproject.org/tpo/core/arti/), a project to
//! implement [Tor](https://www.torproject.org/) in Rust.
//! Many other crates in Arti depend on it, but it should be of general
//! use.

#![deny(missing_docs)]
#![warn(noop_method_call)]
#![deny(unreachable_pub)]
#![deny(clippy::await_holding_lock)]
#![deny(clippy::cargo_common_metadata)]
#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::cognitive_complexity)]
#![deny(clippy::debug_assert_with_mut_call)]
#![deny(clippy::exhaustive_enums)]
#![deny(clippy::exhaustive_structs)]
#![deny(clippy::expl_impl_clone_on_copy)]
#![deny(clippy::fallible_impl_from)]
#![deny(clippy::large_stack_arrays)]
#![warn(clippy::manual_ok_or)]
#![deny(clippy::missing_docs_in_private_items)]
#![warn(clippy::needless_borrow)]
#![warn(clippy::needless_pass_by_value)]
#![warn(clippy::option_option)]
#![warn(clippy::rc_buffer)]
#![deny(clippy::ref_option_ref)]
#![warn(clippy::trait_duplication_in_bounds)]
#![warn(clippy::unseparated_literal_suffix)]

/// An error produced from type derived from type.  These errors can
/// only occur when trying to convert to a type made with caret_enum!
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// Tried to convert to an enumeration type from an integer that
    /// didn't represent a member of that enumeration.
    InvalidInteger,
    /// Tried to convert to an enumeration type from a string that
    /// didn't represent a member of that enumeration.
    InvalidString,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidInteger => write!(f, "Integer was not member of this enumeration"),
            Error::InvalidString => write!(f, "String was not member of this enumeration"),
        }
    }
}

impl std::error::Error for Error {}

/// Declare an integer type with some named elements.
///
/// This macro declares a struct that wraps an integer
/// type, and allows any integer type as a value.  Some values of this type
/// have names, and others do not, but they are all allowed.
///
/// This macro is suitable for protocol implementations that accept
/// any integer on the wire, and have definitions for some of those
/// integers.  For example, Tor cell commands are 8-bit integers, but
/// not every u8 is a currently recognized Tor command.
///
/// # Examples
/// ```
/// use caret::caret_int;
/// caret_int! {
///     pub struct FruitID(u8) {
///         AVOCADO = 7,
///         PERSIMMON = 8,
///         LONGAN = 99
///     }
/// }
///
/// // Known fruits work the way we would expect...
/// let a_num: u8 = FruitID::AVOCADO.into();
/// assert_eq!(a_num, 7);
/// let a_fruit: FruitID = 8.into();
/// assert_eq!(a_fruit, FruitID::PERSIMMON);
/// assert_eq!(format!("I'd like a {}", FruitID::PERSIMMON),
///            "I'd like a PERSIMMON");
///
/// // And we can construct unknown fruits, if we encounter any.
/// let weird_fruit: FruitID = 202.into();
/// assert_eq!(format!("I'd like a {}", weird_fruit),
///            "I'd like a 202");
/// ```
#[macro_export]
macro_rules! caret_int {
    {
       $(#[$meta:meta])*
       $v:vis struct $name:ident ( $numtype:ty ) {
           $(
               $(#[$item_meta:meta])*
               $id:ident = $num:literal
           ),*
           $(,)?
      }
    } => {
        #[derive(PartialEq,Eq,Copy,Clone)]
        $(#[$meta])*
        $v struct $name($numtype);

        impl From<$name> for $numtype {
            fn from(val: $name) -> $numtype { val.0 }
        }
        impl From<$numtype> for $name {
            fn from(num: $numtype) -> $name { $name(num) }
        }
        impl $name {
            $(
                $( #[$item_meta] )*
                pub const $id: $name = $name($num) ; )*
            fn to_str(self) -> Option<&'static str> {
                match self {
                    $( $name::$id => Some(stringify!($id)), )*
                    _ => None,
                }
            }
            /// Return true if this value is one that we recognize.
            $v fn is_recognized(self) -> bool {
                matches!(self,
                         $( $name::$id )|*)
            }
            /// Try to convert this value from one of the recognized names.
            $v fn from_name(name: &str) -> Option<Self> {
                match name {
                    $( stringify!($id) => Some($name::$id), )*
                    _ => None
                }
            }
            /// Return the underlying integer that this value represents.
            fn get(self) -> $numtype {
                self.into()
            }
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self.to_str() {
                    Some(s) => write!(f, "{}", s),
                    None => write!(f, "{}", self.0),
                }
            }
        }
        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({})", stringify!($name), self)
            }
        }
    };

}
