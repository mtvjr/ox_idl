/**********************************************************************************
 * Copyright © 2022 Michael Volling
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the “Software”), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 *********************************************************************************/

use chumsky::prelude::*;

/// The Literal type represents an IDL literal value
///
/// There is only basic support at the moment, with the wchar
/// and wstring types not being yet supported.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Bool(bool),
    Character(char),
    FixedPoint(u64, u64),
    FloatingPoint(f64),
    Integer(u64),
    Str(String),
}

impl Literal {
    /// Builds a parser is able to parse a true boolean literal
    pub fn true_parser() -> impl Parser<char, Literal, Error = Simple<char>> {
        // 7.4.1.3 (19) True values are represented as "TRUE"
        text::keyword("TRUE").map(|_| Literal::Bool(true))
    }

    /// Builds a parser is able to parse a false boolean literal
    pub fn false_parser() -> impl Parser<char, Literal, Error = Simple<char>> {
        // 7.4.1.3 (19) False values are represented as "FALSE"
        text::keyword("FALSE").map(|_| Literal::Bool(false))
    }

    /// Builds a parser is able to parse any boolean literal
    pub fn bool_parser() -> impl Parser<char, Literal, Error = Simple<char>> {
        // 7.4.1.3 (19) <boolean_literal> ::= "TRUE" | "FALSE"
        Self::true_parser().or(Self::false_parser())
    }

    /// Builds a parser is able to parse a decimal integer literal
    pub fn dec_int_parser() -> impl Parser<char, Literal, Error = Simple<char>> {
        // 7.2.6.1
        // An integer literal consisting of a sequence of digits is taken to be decimal
        // (base ten) unless it begins with 0 (digit zero).
        text::int(10).map(|d: String| Literal::Integer(d.parse().unwrap()))
    }

    /// Builds a parser is able to parse a hex integer literal
    pub fn hex_int_parser() -> impl Parser<char, Literal, Error = Simple<char>> {
        // 7.2.6.1
        // A sequence of digits preceded by 0x (or 0X) is taken to be a hexadecimal
        // integer (base sixteen). The hexadecimal digits include a (or A) through
        // f (or F) with decimal values ten through fifteen, respectively.
        just("0x")
            .or(just("0X"))
            .ignore_then(text::int(16))
            .map(|d: String| Literal::Integer(u64::from_str_radix(d.as_str(), 16).unwrap()))
    }

    /// Builds a parser is able to parse a octal integer literal
    pub fn oct_int_parser() -> impl Parser<char, Literal, Error = Simple<char>> {
        // 7.2.6.1
        // A sequence of digits starting with 0 is taken to be an octal integer (base eight).
        // The digits 8 and 9 are not octal digits and thus are not allowed in an octal
        // integer literal.
        just("0").then(text::int(8)).map(|(_p, d): (&str, String)| {
            Literal::Integer(u64::from_str_radix(d.as_str(), 8).unwrap())
        })
    }

    /// Builds a parser is able to parse any integer literal
    pub fn int_parser() -> impl Parser<char, Literal, Error = Simple<char>> {
        // 7.2.6.1
        // An integer literal consisting of a sequence of digits is taken to be decimal
        // (base ten) unless it begins with 0 (digit zero).
        //
        // A sequence of digits starting with 0 is taken to be an octal integer (base eight).
        // The digits 8 and 9 are not octal digits and thus are not allowed in an octal
        // integer literal.
        //
        // A sequence of digits preceded by 0x (or 0X) is taken to be a hexadecimal
        // integer (base sixteen). The hexadecimal digits include a (or A) through
        // f (or F) with decimal values ten through fifteen, respectively.
        Self::hex_int_parser()
            .or(Self::oct_int_parser())
            .or(Self::dec_int_parser())
    }

    /// Builds a parser is able to parse any floating point literal
    pub fn float_parser() -> impl Parser<char, Literal, Error = Simple<char>> {
        // 7.2.6.4
        // A floating-point literal consists of an integer part, a decimal point
        // (.), a fraction part, an e or E, and an optionally signed integer
        // exponent. The integer and fraction parts both consist of a sequence
        // of decimal (base ten) digits. Either the integer part or the fraction
        // part (but not both) may be missing; either the decimal point or the
        // letter e (or E) and the exponent (but not both) may be missing.
        let digits = text::digits(10);
        let dot = just('.');

        let decimal_only = digits
            .then_ignore(dot)
            .map(|d| Self::FloatingPoint(d.parse().unwrap()));

        let fractional_only = dot
            .ignore_then(digits)
            .map(|f| Self::FloatingPoint(('.'.to_string() + f.as_str()).parse().unwrap()));

        let decimal_and_fractional =
            digits
                .then_ignore(just('.'))
                .then(digits)
                .map(|(d, f): (String, String)| {
                    Self::FloatingPoint((d + "." + f.as_str()).parse().unwrap())
                });

        decimal_and_fractional.or(decimal_only).or(fractional_only)
    }

    /// Builds a parser is able to parse a fixed point literal
    pub fn fixed_parser() -> impl Parser<char, Literal, Error = Simple<char>> {
        // 7.2.6.5
        // A fixed-point decimal literal consists of an integer part, a decimal
        // point (.), a fraction part and a d or D. The integer and fraction
        // parts both consist of a sequence of decimal (base 10) digits. Either
        // the integer part or the fraction part (but not both) may be missing;
        // the decimal point (but not the letter d or D) may be missing.
        let digits = text::digits(10);
        let dot = just('.');
        let the_d = just('d').or(just('D'));

        let decimal_only = digits
            .then_ignore(dot.repeated().at_most(1))
            .then_ignore(the_d)
            .map(|d| Self::FixedPoint(d.parse().unwrap(), 0));

        let fractional_only = dot
            .ignore_then(digits)
            .then_ignore(the_d)
            .map(|f| Self::FixedPoint(0, f.parse().unwrap()));

        let decimal_and_fractional = digits
            .then_ignore(just('.'))
            .then(digits)
            .then_ignore(the_d)
            .map(|(d, f): (String, String)| {
                Self::FixedPoint(d.parse().unwrap(), f.parse().unwrap())
            });

        decimal_and_fractional.or(decimal_only).or(fractional_only)
    }

    /// Builds a parser is able to parse a character literal
    pub fn char_parser() -> impl Parser<char, Literal, Error = Simple<char>> {
        // 7.2.6.2
        // A char is an 8-bit quantity with a numerical value between 0 and 255 (decimal).
        // The value of a space, alphabetic, digit, or graphic character literal is the
        // numerical value of the character as defined in the ISO Latin-1 (8859-1)
        // character set standard (see Table 7-2 on page 14, Table 7-3 on page 15 and Table
        // 7-4 on page 16). The value of a null is 0. The value of a formatting character
        // literal is the numerical value of the character as defined in the ISO 646
        // standard (see Table 7-5 on page 17). The meaning of all other characters is
        // implementation-dependent.
        //
        // NOTE: Since ASCII and ISO 646 are the same for 8 bit characters, we should
        // be fine to use 'is_ascii'
        //
        // TODO: Support escape sequences
        filter::<_, _, Simple<char>>(|c: &char| c.is_ascii())
            .delimited_by(just("'"), just("'"))
            .map(|c: char| Self::Character(c))
    }

    /// Builds a parser is able to parse a string literal
    pub fn string_parser() -> impl Parser<char, Literal, Error = Simple<char>> {
        // 7.2.6.3
        // Strings are null-terminated sequences of characters. Strings are of
        // type string if they are made of non-wide characters or wstring
        // (wide string) if they are made of wide characters.
        //
        // A string literal is a sequence of character literals (as defined in
        // 7.2.6.2, Character Literals), with the exception of the character with
        // numeric value 0, surrounded by double quotes, as in:
        //    const string S1 = "Hello";
        //
        // Wide string literals have in addition an L prefix, for example:
        //    const wstring S2 = L"Hello";
        //
        // Both wide and non-wide string literals must be specified using characters
        // from the ISO Latin-1 (8859-1) character set.  A string literal shall not
        // contain the character ‘\0’. A wide string literal shall not contain the
        // wide character with value zero.
        //
        // Adjacent string literals are concatenated. Characters in concatenated strings
        // are kept distinct. For example, "\xA" "B" contains the two characters
        // ‘\xA’ and ‘B’ after concatenation (and not the single hexadecimal character
        // ‘\xAB’).
        //
        // TODO: Support escape sequences
        //
        // FIXME: Right now we are parsing the utf-8 format. Ideally we would use the
        // Latin-1 character set
        let single_string = filter::<_, _, Simple<char>>(|c: &char| *c != '"')
            .repeated()
            .delimited_by(just('"'), just('"'))
            .collect::<String>();

        // Now support implicit concatination
        single_string
            .then_ignore(text::whitespace())
            .repeated()
            .map(|vs| Self::Str(vs.concat()))
    }

    /// Builds a parser is able to parse any literal
    #[allow(dead_code)]
    pub fn parser() -> impl Parser<char, Literal, Error = Simple<char>> {
        Self::bool_parser()
            .or(Self::fixed_parser()) // Fixed needs to be before float
            .or(Self::float_parser()) // Float needs to be before int
            .or(Self::int_parser())
            .or(Self::char_parser())
            .or(Self::string_parser())
    }
}

#[cfg(test)]
mod literal_tests {
    use crate::literal::Literal;
    use chumsky::Parser;

    #[test]
    fn parse_true() {
        let in_str = "TRUE";
        let result = Literal::true_parser().parse(in_str);
        assert_eq!(result, Ok(Literal::Bool(true)));
    }

    #[test]
    fn parse_false() {
        let in_str = "FALSE";
        let result = Literal::false_parser().parse(in_str);
        assert_eq!(result, Ok(Literal::Bool(false)));
    }

    #[test]
    fn parse_bool() {
        let true_str = "TRUE";
        let false_str = "FALSE";

        assert_eq!(
            Ok(Literal::Bool(true)),
            Literal::bool_parser().parse(true_str)
        );
        assert_eq!(
            Ok(Literal::Bool(false)),
            Literal::bool_parser().parse(false_str)
        );
    }

    #[test]
    fn parse_dec_int() {
        assert_eq!(
            Literal::dec_int_parser().parse("1234"),
            Ok(Literal::Integer(1234))
        );
        assert_eq!(
            Literal::dec_int_parser().parse("9876543210"),
            Ok(Literal::Integer(9876543210))
        );
    }

    #[test]
    fn parse_hex_int() {
        assert_eq!(
            Literal::hex_int_parser().parse("0x1234"),
            Ok(Literal::Integer(0x1234))
        );
        assert_eq!(
            Literal::hex_int_parser().parse("0xDEADBEEF"),
            Ok(Literal::Integer(0xDEADBEEF))
        );
        assert_eq!(
            Literal::hex_int_parser().parse("0xdeadbeef"),
            Ok(Literal::Integer(0xDEADBEEF))
        );
        assert_eq!(
            Literal::hex_int_parser().parse("0Xdeadbeef"),
            Ok(Literal::Integer(0xDEADBEEF))
        );
    }

    #[test]
    fn parse_oct_int() {
        assert_eq!(
            Literal::oct_int_parser().parse("01234"),
            Ok(Literal::Integer(668))
        );
        assert_eq!(
            Literal::oct_int_parser().parse("0527"),
            Ok(Literal::Integer(343))
        );
    }

    #[test]
    fn parse_int() {
        // Decimal
        assert_eq!(
            Literal::int_parser().parse("1234"),
            Ok(Literal::Integer(1234))
        );
        assert_eq!(
            Literal::int_parser().parse("9876543210"),
            Ok(Literal::Integer(9876543210))
        );

        // Hex
        assert_eq!(
            Literal::int_parser().parse("0x1234"),
            Ok(Literal::Integer(0x1234))
        );
        assert_eq!(
            Literal::int_parser().parse("0xDEADBEEF"),
            Ok(Literal::Integer(0xDEADBEEF))
        );
        assert_eq!(
            Literal::int_parser().parse("0xdeadbeef"),
            Ok(Literal::Integer(0xDEADBEEF))
        );
        assert_eq!(
            Literal::int_parser().parse("0Xdeadbeef"),
            Ok(Literal::Integer(0xDEADBEEF))
        );

        // Octal
        assert_eq!(
            Literal::oct_int_parser().parse("01234"),
            Ok(Literal::Integer(668))
        );
        assert_eq!(
            Literal::oct_int_parser().parse("0527"),
            Ok(Literal::Integer(343))
        );
    }

    #[test]
    fn parse_float() {
        assert_eq!(
            Literal::float_parser().parse("1.1"),
            Ok(Literal::FloatingPoint(1.1))
        );
        assert_eq!(
            Literal::float_parser().parse("19234.12534"),
            Ok(Literal::FloatingPoint(19234.12534))
        );
        assert_eq!(
            Literal::float_parser().parse("0."),
            Ok(Literal::FloatingPoint(0.0))
        );
        assert_eq!(
            Literal::float_parser().parse(".0"),
            Ok(Literal::FloatingPoint(0.0))
        );
        assert_eq!(
            Literal::float_parser().parse("0.0"),
            Ok(Literal::FloatingPoint(0.0))
        );
        assert!(Literal::float_parser().parse(".").is_err());
    }

    #[test]
    fn parse_char() {
        assert_eq!(
            Literal::char_parser().parse("'3'"),
            Ok(Literal::Character('3'))
        );
        assert_eq!(
            Literal::char_parser().parse("'A'"),
            Ok(Literal::Character('A'))
        );
        assert_eq!(
            Literal::char_parser().parse("'a'"),
            Ok(Literal::Character('a'))
        );
    }

    #[test]
    fn parse_string() {
        // Test normal strings
        let validate_string = |s: &str| {
            assert_eq!(
                Literal::string_parser().parse(("\"".to_string() + s + "\"").as_str()),
                Ok(Literal::Str(s.to_string()))
            );
        };
        validate_string("Hello World!");
        validate_string("");
        validate_string("I ate a beef sandwich");

        // Test implicit concatination
        assert_eq!(
            Literal::string_parser().parse("\"Hello\" \"World\""),
            Ok(Literal::Str("HelloWorld".to_string()))
        );
    }

    #[test]
    fn parse_fixed() {
        assert_eq!(
            Literal::fixed_parser().parse("3.6D"),
            Ok(Literal::FixedPoint(3, 6))
        );
        assert_eq!(
            Literal::fixed_parser().parse("1.2d"),
            Ok(Literal::FixedPoint(1, 2))
        );
        assert_eq!(
            Literal::fixed_parser().parse(".3d"),
            Ok(Literal::FixedPoint(0, 3))
        );
        assert_eq!(
            Literal::fixed_parser().parse("3d"),
            Ok(Literal::FixedPoint(3, 0))
        );
    }

    #[test]
    fn parse_literal() {
        let p = Literal::parser();

        assert_eq!(
            p.parse("\"String\""),
            Ok(Literal::Str("String".to_string()))
        );
        assert_eq!(p.parse("'c'"), Ok(Literal::Character('c')));
        assert_eq!(p.parse("2.1"), Ok(Literal::FloatingPoint(2.1)));
        assert_eq!(p.parse("2.1d"), Ok(Literal::FixedPoint(2, 1)));
        assert_eq!(p.parse("TRUE"), Ok(Literal::Bool(true)));
        assert_eq!(p.parse("3"), Ok(Literal::Integer(3)));
    }
}
