//! # The ECMA 402 API surface proposal for rust
//!
//! This proposal contains traits declarations for a rust-flavored [ECMA402
//! implementation](https://www.ecma-international.org/publications/standards/Ecma-402.htm).
//!
//! All ECMA402 functions require specifying at least one, if not multiple locales
//! for which they will then return appropriate results. For this reason, this proposal shows a
//! minimal type signature required for something like that to be feasible.
//!
//! The idea of this proposal is to define common API surface that would admit different
//! implementations of ECMA 402 inspired libraries.  The existence of a common standard would allow
//! drop-in replacement of say [ICU-based]() implementation for
//! [Unic-based](https://crates.io/crates/unic) implementation at some future, which is relevant
//! for users that need the Unicode support functionality today, and are not prepared to wait until
//! Unic conquers the rust world.
//!
//! See [LanguageIdentifier] for an example of such a trait.
//!
//! ## A note about presentation
//!
//! This proposal is deliberately written in the form of compilable rust code, and is perhaps best
//! consumed by looking at the output of the command `cargo doc --open` ran at the top level
//! directory.  It's not quite [literate
//! programming](https://en.wikipedia.org/wiki/Literate_programming) but should be close enough for
//! our purpose.  And our purpose here is to present the API alongside a glimpse of how it would be
//! used.
//!
//! The proposed APIs are quickly tested with implementations given in the `mod tests` section of
//! the source code.  I originally put those into doc-tests, but doc-tests were difficult to write
//! efficiently so I rolled them into a separate test module, as is customary in rust.
//!
//! # Part 1: Language identifiers and BCP 47 representation
//!
//! This proposal contains the following traits:
//!
//! * [AsBCP47]: A single-method trait for converting an object into a BCP 47 serialized form.
//!   This is a minimum required to be able to define ECMA402 compatible APIs, which take arrays
//!   of locales and friends.
//! * [LanguageIdentifier]: Adds immutable getters for language identifier components.

/// Represents an immutable language identifier.
///
/// This trait can be passed into functions that are not expected to be able to mutate the
/// identifier.  The `language` property must be defined, or equal to the literal string `und` if
/// it is left unspecified.  Other properties are optional.  See [weird::Variants] for the
/// obviously missing treatment of variants subtags.
pub trait LanguageIdentifier {
    /// Returns the language subtag of the `language::Identifier`.  If the
    /// language subtag is empty, the returned value is `und`.
    fn language(&self) -> &str;

    /// Returns the region subtag of the `language::Identifier`, if one is set.
    fn region(&self) -> Option<&str>;

    /// Returns the script subtag of the `language::Identifier`, if one is set.
    fn script(&self) -> Option<&str>;
}

/// Traits that ended up being unusual or weird because of issues unrelated to their structure.
/// Specifically [weird::Variants] departs from what it should have been because of issues with
/// defining a lifetime of an iterator.
pub mod weird {

    /// Variants allow iteration over variants, regardless of whether they are returned as owned or
    /// not.  See `mod tests` below for example implementations.  [Variants] needs to be
    /// implemented on a reference to the container type, not the type itself, in order for the
    /// iterator to have the correct lifetime.
    pub trait Variants {
        /// The type of the item yieled by the iterator returned by [Variants::variants].  Note
        /// that [Variants::Item] may be a reference to the type stored in the iterator.  See tests
        /// for the details.
        type Item;
        /// The type of the iterator returned by [Variants::variants].
        type Iter: ExactSizeIterator<Item = Self::Item>;
        fn variants(self) -> Self::Iter;
    }
}

/// Allows representing the item (a locale object or a language identifier) in the form compatible
/// with the [BCP 47 representation](https://tools.ietf.org/html/bcp47).
pub trait AsBCP47 {
    /// Returns a BCP 47 representation of the object.  This represents a canonical serialization
    /// of all properties of a language identifier or a locale into a string.  Some objects, like
    /// full-blown locales have extensions that are required to be serialized in a very specific
    /// way.  Follow BCP 47 practices to do so when implementing this trait.
    fn as_bcp47(&self) -> &str;
}

/// This trait corresponds to the `Intl` object of ECMA-402.
pub trait Intl {
    /// Canonicalizes all locale names that were passed in.
    fn get_canonical_locales(&self, locales: &Vec<impl AsRef<[u8]>>) -> Vec<String>;
}

#[cfg(test)]
mod tests {
    use crate::{LanguageIdentifier, weird::Variants};

    /// Here's an example struct that implements [LanguageIdentifier] and [Variants] traits, and
    /// borrows all of its constituent elements.  Also, fields can't have the same names as
    /// trait methods.
    #[derive(Debug)]
    struct BorrowedId {
        lang: &'static str,
        reg: Option<&'static str>,
        scr: Option<&'static str>,
        var: Vec<&'static str>,
    }
    impl LanguageIdentifier for BorrowedId {
        fn language(&self) -> &str {
            self.lang
        }
        fn region(&self) -> Option<&str> {
            self.reg
        }
        fn script(&self) -> Option<&str> {
            self.scr
        }
    }
    impl<'a> Variants for &'a BorrowedId {
        type Item = &'a &'a str;
        type Iter = std::slice::Iter<'a, &'a str>;
        fn variants(self) -> Self::Iter {
            self.var.iter()
        }
    }

    #[test]
    fn borrowed_tests() {
        let id = BorrowedId {
            lang: "en",
            reg: Some("US"),
            scr: None,
            // Note the variants in this example are not valid.
            var: vec!["east_coast", "west_coast"],
        };
        assert_eq!(id.language(), "en");
        assert_eq!(id.region(), Some("US"));
        assert_eq!(id.script(), None);
        assert_eq!(id.variants().next().unwrap(), &"east_coast");
        assert_eq!(
            id.variants()
                .map(|v| v.to_owned().to_owned())
                .collect::<Vec<String>>(),
            vec!["east_coast", "west_coast"]
        );
    }

    /// Here's an example struct that implements [LanguageIdentifier] and [Variants] traits, and
    /// owns all its constituent elements.
    struct OwnedId {
        lang: String,
        reg: Option<String>,
        scr: Option<String>,
        var: Vec<String>,
    }
    impl LanguageIdentifier for OwnedId {
        fn language(&self) -> &str {
            &self.lang
        }
        fn region(&self) -> Option<&str> {
            self.reg.as_deref()
        }
        fn script(&self) -> Option<&str> {
            self.scr.as_deref()
        }
    }
    impl<'a> Variants for &'a OwnedId {
        type Item = &'a String;
        type Iter = std::slice::Iter<'a, String>;
        fn variants(self) -> Self::Iter {
            self.var.iter()
        }
    }

    #[test]
    fn owned_tests() {
        let id = OwnedId {
            lang: "en".to_string(),
            reg: Some("US".to_string()),
            scr: None,
            // Note the variants in this example are not valid.
            var: vec!["east_coast".to_string(), "west_coast".to_string()],
        };
        assert_eq!(id.language(), "en");
        assert_eq!(id.region(), Some("US"));
        assert_eq!(id.script(), None);
        assert_eq!(
            id.variants()
                .map(|v| v.to_owned())
                .collect::<Vec<String>>(),
            vec!["east_coast", "west_coast"]
        );
    }


    use crate::Intl;

    struct IntlImpl {}

    impl Intl for IntlImpl {
        // This is a fake implementation that just illustrates how locales get
        // transformed by passing through the filter.  Locales may be non-utf8, which is
        // why the method admits anything that can be represented as a sequence of bytes.
        fn get_canonical_locales(&self, locales: &Vec<impl AsRef<[u8]>>) -> Vec<String> {
            locales
                .iter()
                // A real library would not enforce UTF-8, but would consider the possibility that
                // the locale passed in is using a different encoding than UTF-8.
                .map(|l| std::str::from_utf8(l.as_ref()).expect("can not be converted to utf8"))
                // Shows how locales can be omitted from the result.
                .filter(|l| *l != "skip")
                .map(|l| format!("canonicalized({})", l))
                .collect::<Vec<String>>()
        }
    }

    #[test]
    fn test_canonical_locales() {
        let i = IntlImpl {};
        let c = i.get_canonical_locales(&vec!["en-us", "skip", "fr-fr"]);
        assert_eq!(c, vec!["canonicalized(en-us)", "canonicalized(fr-fr)"]);
    }
}
