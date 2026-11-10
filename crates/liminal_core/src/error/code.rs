//! [`ErrorCode`]: the stable error code `LIM-<AREA>-<NNNN>` (ADR-0017).

use super::CoreError;

/// Longest area, in bytes.
const AREA_MAX: usize = 8;
/// Largest number.
const NUMBER_MAX: u16 = 9999;

/// A stable error code `LIM-<AREA>-<NNNN>` (ADR-0017).
///
/// `AREA` is the `req_prefix` of the spec of the crate that defines the error: 1 to 8 ASCII
/// uppercase letters. `NNNN` is a number from 1 to 9999, written with exactly four digits. A code
/// is never reused or renumbered. Every value of this type is a valid code; only [`ErrorCode::new`]
/// and [`ErrorCode::parse`] create values.
///
/// `Copy`, 12 bytes. Ordered by area (byte-wise lexicographic), then by number.
///
/// # Examples
///
/// ```
/// use liminal_core::error::ErrorCode;
///
/// // Checked at compile time: an invalid area or number here is a compile error.
/// const NOT_REGISTERED: ErrorCode = ErrorCode::new("ECS", 3);
///
/// assert_eq!(NOT_REGISTERED.to_string(), "LIM-ECS-0003");
/// assert_eq!(NOT_REGISTERED.area(), "ECS");
/// assert_eq!(NOT_REGISTERED.number(), 3);
/// assert_eq!(ErrorCode::parse("LIM-ECS-0003"), Ok(NOT_REGISTERED));
/// assert!(ErrorCode::parse("LIM-ecs-3").is_err());
/// ```
///
/// An invalid code in a `const` item does not compile:
///
/// ```compile_fail
/// use liminal_core::error::ErrorCode;
/// const BAD: ErrorCode = ErrorCode::new("ecs", 3);
/// ```
// Field order matters: the derived `Ord` compares `area` (zero-padded, so byte-wise
// lexicographic with a prefix ordered first), then `area_len` (equal when the areas are equal),
// then `number`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ErrorCode {
    /// The area's bytes, followed by zero bytes.
    area: [u8; AREA_MAX],
    /// Number of area bytes, 1..=8.
    area_len: u8,
    /// 1..=9999.
    number: u16,
}

impl ErrorCode {
    /// Creates the code `LIM-<area>-<number>`, for constants.
    ///
    /// # Panics
    ///
    /// Panics if `area` is not 1 to 8 ASCII uppercase letters (`A`–`Z`) or `number` is not in
    /// 1..=9999. Evaluated in a const context (a `const` item or a `const { }` block) the panic is
    /// a compile error, so define codes there. For text that is not known at compile time, use
    /// [`ErrorCode::parse`].
    #[must_use]
    pub const fn new(area: &'static str, number: u16) -> ErrorCode {
        match Self::from_parts(area.as_bytes(), number) {
            Some(code) => code,
            None => panic!(
                "invalid error code: the area must be 1 to 8 ASCII uppercase letters and the number 1 to 9999"
            ),
        }
    }

    /// Parses the text form, for example `LIM-ECS-0003`.
    ///
    /// Accepts exactly: `LIM-`, 1 to 8 ASCII uppercase letters, `-`, and 4 ASCII digits whose value
    /// is 1 to 9999. No surrounding whitespace or other characters.
    ///
    /// # Errors
    ///
    /// [`CoreError::InvalidErrorCode`] (LIM-CORE-0001) with the input as `text`, for every other
    /// input.
    pub fn parse(text: &str) -> Result<ErrorCode, CoreError> {
        Self::parse_parts(text).ok_or_else(|| CoreError::InvalidErrorCode {
            text: text.to_string(),
        })
    }

    /// The area, for example `ECS`: 1 to 8 ASCII uppercase letters.
    ///
    /// # Panics
    ///
    /// Only if the type invariant is broken (a bug in this crate): the stored area is always
    /// ASCII letters.
    #[must_use]
    pub fn area(&self) -> &str {
        let bytes = &self.area[..usize::from(self.area_len)];
        core::str::from_utf8(bytes).expect("invariant: an error code's area is ASCII letters")
    }

    /// The number, 1..=9999.
    #[must_use]
    pub fn number(&self) -> u16 {
        self.number
    }

    /// The code with this area and number, or `None` if either is invalid.
    const fn from_parts(area: &[u8], number: u16) -> Option<ErrorCode> {
        if area.is_empty() || area.len() > AREA_MAX || number == 0 || number > NUMBER_MAX {
            return None;
        }
        let mut bytes = [0_u8; AREA_MAX];
        let mut area_len = 0_u8;
        let mut i = 0;
        while i < area.len() {
            if !area[i].is_ascii_uppercase() {
                return None;
            }
            bytes[i] = area[i];
            area_len += 1;
            i += 1;
        }
        Some(ErrorCode {
            area: bytes,
            area_len,
            number,
        })
    }

    fn parse_parts(text: &str) -> Option<ErrorCode> {
        let rest = text.strip_prefix("LIM-")?;
        let (area, digits) = rest.split_once('-')?;
        if digits.len() != 4 {
            return None;
        }
        let mut number = 0_u16;
        for digit in digits.bytes() {
            if !digit.is_ascii_digit() {
                return None;
            }
            number = number * 10 + u16::from(digit - b'0');
        }
        Self::from_parts(area.as_bytes(), number)
    }
}

/// Writes `LIM-`, the area, `-`, and the number zero-padded to 4 digits: `LIM-ECS-0003`.
impl core::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "LIM-{}-{:04}", self.area(), self.number)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn invalid(text: &str) -> Result<ErrorCode, CoreError> {
        Err(CoreError::InvalidErrorCode {
            text: text.to_string(),
        })
    }

    /// Verifies: CORE-R001
    #[test]
    fn parse_accepts_the_grammar_boundaries() {
        for (text, area, number) in [
            ("LIM-A-0001", "A", 1),
            ("LIM-ABCDEFGH-9999", "ABCDEFGH", 9999),
            ("LIM-ECS-0003", "ECS", 3),
            ("LIM-Z-0010", "Z", 10),
            ("LIM-LIM-1000", "LIM", 1000),
        ] {
            let code = ErrorCode::parse(text).unwrap();
            assert_eq!((code.area(), code.number()), (area, number), "{text}");
        }
    }

    /// Verifies: CORE-R001
    #[test]
    fn parse_rejects_everything_else_with_the_input_as_text() {
        for text in [
            "",
            "LIM",
            "LIM-",
            "LIM--0001",
            "LIM-ECS",
            "LIM-ECS-",
            "LIM-ECS-0000",
            "LIM-ECS-1",
            "LIM-ECS-001",
            "LIM-ECS-00001",
            "LIM-ECS-10000",
            "LIM-ECS-+001",
            "LIM-ECS--001",
            "LIM-ECS-00a1",
            "LIM-ECS-0001 ",
            " LIM-ECS-0001",
            "LIM-ECS-0001\n",
            "LIM-ABCDEFGHI-0001",
            "LIM-ecs-0001",
            "LIM-EcS-0001",
            "LIM-E1S-0001",
            "LIM-E_S-0001",
            "LIM-E-S-0001",
            "lim-ECS-0001",
            "LIM_ECS_0001",
            "XLIM-ECS-0001",
            "LIM-ÉCS-0001",
            "LIM-ECS-000١",
            "LIM-ECS-٠٠٠١",
            "LIM-ECS-0001-0001",
        ] {
            assert_eq!(ErrorCode::parse(text), invalid(text), "{text:?}");
        }
    }

    /// Verifies: CORE-R002
    #[test]
    fn display_pads_the_number_and_round_trips() {
        for (area, number, text) in [
            ("A", 1, "LIM-A-0001"),
            ("CORE", 10, "LIM-CORE-0010"),
            ("XTASK", 123, "LIM-XTASK-0123"),
            ("ABCDEFGH", 9999, "LIM-ABCDEFGH-9999"),
        ] {
            let code = ErrorCode::new(area, number);
            assert_eq!(code.to_string(), text);
            assert_eq!(ErrorCode::parse(text), Ok(code));
            assert_eq!((code.area(), code.number()), (area, number));
        }
    }

    /// Verifies: CORE-R003
    #[test]
    fn new_is_usable_in_const_items_and_equals_parse() {
        const CODE: ErrorCode = ErrorCode::new("ECS", 3);
        const LONGEST: ErrorCode = ErrorCode::new("ABCDEFGH", 9999);
        assert_eq!(Ok(CODE), ErrorCode::parse("LIM-ECS-0003"));
        assert_eq!(Ok(LONGEST), ErrorCode::parse("LIM-ABCDEFGH-9999"));
        assert_eq!(Ok(ErrorCode::new("A", 1)), ErrorCode::parse("LIM-A-0001"));
    }

    /// Verifies: CORE-R003
    #[test]
    fn new_panics_on_an_invalid_area_or_number() {
        for (area, number) in [
            ("", 1),
            ("ABCDEFGHI", 1),
            ("ecs", 1),
            ("E1", 1),
            ("E-S", 1),
            ("É", 1),
            ("ECS ", 1),
            ("ECS", 0),
            ("ECS", 10_000),
            ("ECS", u16::MAX),
        ] {
            let result = std::panic::catch_unwind(|| ErrorCode::new(area, number));
            assert!(result.is_err(), "{area:?} {number} did not panic");
        }
    }

    /// Verifies: CORE-R004
    #[test]
    fn code_is_copy_and_at_most_16_bytes() {
        fn assert_copy<T: Copy>() {}
        assert_copy::<ErrorCode>();
        assert!(size_of::<ErrorCode>() <= 16);
        assert_eq!(size_of::<ErrorCode>(), 12);
    }

    /// Verifies: CORE-R004
    #[test]
    fn order_is_by_area_bytes_then_number() {
        // Listed in ascending order: a prefix sorts before its extensions, the area decides
        // before the number.
        let ascending = [
            ErrorCode::new("A", 1),
            ErrorCode::new("A", 9999),
            ErrorCode::new("AA", 1),
            ErrorCode::new("AB", 2),
            ErrorCode::new("ABCDEFGH", 1),
            ErrorCode::new("B", 1),
            ErrorCode::new("B", 2),
            ErrorCode::new("CORE", 1),
            ErrorCode::new("ECS", 1),
            ErrorCode::new("Z", 1),
        ];
        for (i, a) in ascending.iter().enumerate() {
            for (j, b) in ascending.iter().enumerate() {
                assert_eq!(a.cmp(b), i.cmp(&j), "{a} vs {b}");
                assert_eq!(a.partial_cmp(b), Some(i.cmp(&j)));
                // The order is the order of (area, number).
                assert_eq!(
                    a.cmp(b),
                    (a.area().as_bytes(), a.number()).cmp(&(b.area().as_bytes(), b.number()))
                );
            }
        }
    }
}
