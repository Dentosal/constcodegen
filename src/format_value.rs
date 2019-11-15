use serde::Deserialize;

use crate::value::Primitive;

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Format {
    pub boolean: Option<BooleanFormat>,
    pub integer: Option<IntegerFormat>,
}
impl Format {
    pub fn format(&self, value: &Primitive) -> String {
        (match value {
            Primitive::Boolean(v) => self.boolean.clone().map(|b| b.format(*v)),
            Primitive::Integer(v) => self.integer.clone().map(|b| b.format(*v)),
            _ => None,
        })
        .unwrap_or_else(|| value.to_string())
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BooleanFormat {
    #[serde(rename = "true")]
    true_: String,
    #[serde(rename = "false")]
    false_: String,
}
impl BooleanFormat {
    pub fn format(&self, boolean: bool) -> String {
        if boolean {
            self.true_.clone()
        } else {
            self.false_.clone()
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct IntegerFormat {
    radix: Radix,
    /// Underscore between every n digits, zero to disable
    underscores: u8,
    /// Zero pad to n digits, zero to disable
    zero_pad: u8,
    /// Omit `0x` prefix on non-base 10 numbers
    omit_prefix: bool,
}
impl IntegerFormat {
    pub fn format(&self, mut integer: i128) -> String {
        let negative: bool = integer < 0;
        let radix = self.radix.value();

        integer = integer.abs();
        let mut digits: Vec<char> = Vec::new();
        while integer > 0 {
            let digit = (integer % (radix as i128)) as u32;
            digits.push(std::char::from_digit(digit, radix).unwrap());
            integer /= radix as i128;
        }

        while digits.len() < (self.zero_pad as usize) {
            digits.push('0');
        }

        let mut result: String = digits.into_iter().rev().collect();
        if self.underscores != 0 {
            let mut i = result.len();
            while i > self.underscores as usize {
                i -= self.underscores as usize;
                result.insert(i, '_');
            }
        }

        if !self.omit_prefix {
            result = format!("{}{}", self.radix.prefix(), result);
        }

        if negative {
            format!("-{}", result)
        } else {
            result
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub enum Radix {
    #[serde(alias = "bin")]
    Binary,
    #[serde(alias = "oct")]
    Octal,
    #[serde(alias = "dec")]
    Decimal,
    #[serde(alias = "hex")]
    Hexadecimal,
}
impl Radix {
    fn value(self) -> u32 {
        match self {
            Self::Binary => 2,
            Self::Octal => 8,
            Self::Decimal => 10,
            Self::Hexadecimal => 16,
        }
    }

    fn prefix(self) -> &'static str {
        match self {
            Self::Binary => "0b",
            Self::Octal => "0o",
            Self::Decimal => "",
            Self::Hexadecimal => "0x",
        }
    }
}
impl Default for Radix {
    fn default() -> Self {
        Self::Decimal
    }
}

#[cfg(test)]
mod test_formatting {
    use super::*;

    #[test]
    fn test_integer_format_hex() {
        let f = IntegerFormat {
            radix: Radix::Hexadecimal,
            ..Default::default()
        };
        assert_eq!(f.format(0x1234), "0x1234");
        assert_eq!(f.format(0x1234_5678), "0x12345678");
        assert_eq!(f.format(0x1234_5678_90ab_cdef), "0x1234567890abcdef");

        let f = IntegerFormat {
            radix: Radix::Hexadecimal,
            underscores: 4,
            ..Default::default()
        };
        assert_eq!(f.format(0x1234), "0x1234");
        assert_eq!(f.format(0x1234_5678), "0x1234_5678");
        assert_eq!(f.format(0x1234_5678_90ab_cdef), "0x1234_5678_90ab_cdef");
        assert_eq!(f.format(0x123_4567), "0x123_4567");

        let f = IntegerFormat {
            radix: Radix::Hexadecimal,
            underscores: 4,
            zero_pad: 8,
            ..Default::default()
        };
        assert_eq!(f.format(0x1234), "0x0000_1234");
        assert_eq!(f.format(0x1234_5678), "0x1234_5678");
        assert_eq!(f.format(0x1234_5678_90ab_cdef), "0x1234_5678_90ab_cdef");
    }

    #[test]
    fn test_integer_format_bin() {
        let f = IntegerFormat {
            radix: Radix::Binary,
            ..Default::default()
        };
        assert_eq!(f.format(0b1010), "0b1010");

        let f = IntegerFormat {
            radix: Radix::Binary,
            underscores: 4,
            ..Default::default()
        };
        assert_eq!(f.format(0b1010), "0b1010");
        assert_eq!(f.format(0b1010_0101), "0b1010_0101");
        assert_eq!(f.format(0b1111_0000_1100_0011), "0b1111_0000_1100_0011");

        let f = IntegerFormat {
            radix: Radix::Binary,
            underscores: 4,
            zero_pad: 8,
            ..Default::default()
        };
        assert_eq!(f.format(0b1010), "0b0000_1010");
        assert_eq!(f.format(0b1010_0101), "0b1010_0101");
        assert_eq!(f.format(0b1111_0000_1100_0011), "0b1111_0000_1100_0011");
    }
}
