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
    pub fn true_parser() -> impl Parser<char, Literal, Error = Simple<char>> {
        text::keyword("TRUE").map(|_| Literal::Bool(true))
    }

    pub fn false_parser() -> impl Parser<char, Literal, Error = Simple<char>> {
        text::keyword("FALSE").map(|_| Literal::Bool(false))
    }

    pub fn bool_parser() -> impl Parser<char, Literal, Error = Simple<char>> {
        Self::true_parser().or(Self::false_parser())
    }

    pub fn dec_int_parser() -> impl Parser<char, Literal, Error = Simple<char>> {
        text::int(10).map(|d: String| Literal::Integer(d.parse().unwrap()))
    }

    pub fn hex_int_parser() -> impl Parser<char, Literal, Error = Simple<char>> {
        just("0x")
            .or(just("0X"))
            .ignore_then(text::int(16))
            .map(|d: String| Literal::Integer(u64::from_str_radix(d.as_str(), 16).unwrap()))
    }

    pub fn oct_int_parser() -> impl Parser<char, Literal, Error = Simple<char>> {
        just("0").then(text::int(8)).map(|(_p, d): (&str, String)| {
            Literal::Integer(u64::from_str_radix(d.as_str(), 8).unwrap())
        })
    }

    pub fn int_parser() -> impl Parser<char, Literal, Error = Simple<char>> {
        // Ensure that hex is before oct and that oct is before dec to prevent misreads
        Self::hex_int_parser()
            .or(Self::oct_int_parser())
            .or(Self::dec_int_parser())
    }

    pub fn float_parser() -> impl Parser<char, Literal, Error = Simple<char>> {
        // TODO: Support e notation
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

    pub fn fixed_parser() -> impl Parser<char, Literal, Error = Simple<char>> {
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

    pub fn char_parser() -> impl Parser<char, Literal, Error = Simple<char>> {
        // TODO: Support escape sequences
        filter::<_, _, Simple<char>>(|c: &char| c.is_ascii())
            .delimited_by(just("'"), just("'"))
            .map(|c: char| Self::Character(c))
    }

    pub fn string_parser() -> impl Parser<char, Literal, Error = Simple<char>> {
        // TODO: Support escape sequences
        // FIXME: This should ideally use the Latin-1 character set
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
    use chumsky::Parser;

    #[test]
    fn parse_true() {
        let in_str = "TRUE";
        let result = crate::Literal::true_parser().parse(in_str);
        assert_eq!(result, Ok(crate::Literal::Bool(true)));
    }

    #[test]
    fn parse_false() {
        let in_str = "FALSE";
        let result = crate::Literal::false_parser().parse(in_str);
        assert_eq!(result, Ok(crate::Literal::Bool(false)));
    }

    #[test]
    fn parse_bool() {
        let true_str = "TRUE";
        let false_str = "FALSE";

        assert_eq!(
            Ok(crate::Literal::Bool(true)),
            crate::Literal::bool_parser().parse(true_str)
        );
        assert_eq!(
            Ok(crate::Literal::Bool(false)),
            crate::Literal::bool_parser().parse(false_str)
        );
    }

    #[test]
    fn parse_dec_int() {
        assert_eq!(
            crate::Literal::dec_int_parser().parse("1234"),
            Ok(crate::Literal::Integer(1234))
        );
        assert_eq!(
            crate::Literal::dec_int_parser().parse("9876543210"),
            Ok(crate::Literal::Integer(9876543210))
        );
    }

    #[test]
    fn parse_hex_int() {
        assert_eq!(
            crate::Literal::hex_int_parser().parse("0x1234"),
            Ok(crate::Literal::Integer(0x1234))
        );
        assert_eq!(
            crate::Literal::hex_int_parser().parse("0xDEADBEEF"),
            Ok(crate::Literal::Integer(0xDEADBEEF))
        );
        assert_eq!(
            crate::Literal::hex_int_parser().parse("0xdeadbeef"),
            Ok(crate::Literal::Integer(0xDEADBEEF))
        );
        assert_eq!(
            crate::Literal::hex_int_parser().parse("0Xdeadbeef"),
            Ok(crate::Literal::Integer(0xDEADBEEF))
        );
    }

    #[test]
    fn parse_oct_int() {
        assert_eq!(
            crate::Literal::oct_int_parser().parse("01234"),
            Ok(crate::Literal::Integer(668))
        );
        assert_eq!(
            crate::Literal::oct_int_parser().parse("0527"),
            Ok(crate::Literal::Integer(343))
        );
    }

    #[test]
    fn parse_int() {
        // Decimal
        assert_eq!(
            crate::Literal::int_parser().parse("1234"),
            Ok(crate::Literal::Integer(1234))
        );
        assert_eq!(
            crate::Literal::int_parser().parse("9876543210"),
            Ok(crate::Literal::Integer(9876543210))
        );

        // Hex
        assert_eq!(
            crate::Literal::int_parser().parse("0x1234"),
            Ok(crate::Literal::Integer(0x1234))
        );
        assert_eq!(
            crate::Literal::int_parser().parse("0xDEADBEEF"),
            Ok(crate::Literal::Integer(0xDEADBEEF))
        );
        assert_eq!(
            crate::Literal::int_parser().parse("0xdeadbeef"),
            Ok(crate::Literal::Integer(0xDEADBEEF))
        );
        assert_eq!(
            crate::Literal::int_parser().parse("0Xdeadbeef"),
            Ok(crate::Literal::Integer(0xDEADBEEF))
        );

        // Octal
        assert_eq!(
            crate::Literal::oct_int_parser().parse("01234"),
            Ok(crate::Literal::Integer(668))
        );
        assert_eq!(
            crate::Literal::oct_int_parser().parse("0527"),
            Ok(crate::Literal::Integer(343))
        );
    }

    #[test]
    fn parse_float() {
        assert_eq!(
            crate::Literal::float_parser().parse("1.1"),
            Ok(crate::Literal::FloatingPoint(1.1))
        );
        assert_eq!(
            crate::Literal::float_parser().parse("19234.12534"),
            Ok(crate::Literal::FloatingPoint(19234.12534))
        );
        assert_eq!(
            crate::Literal::float_parser().parse("0."),
            Ok(crate::Literal::FloatingPoint(0.0))
        );
        assert_eq!(
            crate::Literal::float_parser().parse(".0"),
            Ok(crate::Literal::FloatingPoint(0.0))
        );
        assert_eq!(
            crate::Literal::float_parser().parse("0.0"),
            Ok(crate::Literal::FloatingPoint(0.0))
        );
        assert!(crate::Literal::float_parser().parse(".").is_err());
    }

    #[test]
    fn parse_char() {
        assert_eq!(
            crate::Literal::char_parser().parse("'3'"),
            Ok(crate::Literal::Character('3'))
        );
        assert_eq!(
            crate::Literal::char_parser().parse("'A'"),
            Ok(crate::Literal::Character('A'))
        );
        assert_eq!(
            crate::Literal::char_parser().parse("'a'"),
            Ok(crate::Literal::Character('a'))
        );
    }

    #[test]
    fn parse_string() {
        // Test normal strings
        let validate_string = |s: &str| {
            assert_eq!(
                crate::Literal::string_parser().parse(("\"".to_string() + s + "\"").as_str()),
                Ok(crate::Literal::Str(s.to_string()))
            );
        };
        validate_string("Hello World!");
        validate_string("");
        validate_string("I ate a beef sandwich");

        // Test implicit concatination
        assert_eq!(
            crate::Literal::string_parser().parse("\"Hello\" \"World\""),
            Ok(crate::Literal::Str("HelloWorld".to_string()))
        );
    }

    #[test]
    fn parse_fixed() {
        assert_eq!(
            crate::Literal::fixed_parser().parse("3.6D"),
            Ok(crate::Literal::FixedPoint(3, 6))
        );
        assert_eq!(
            crate::Literal::fixed_parser().parse("1.2d"),
            Ok(crate::Literal::FixedPoint(1, 2))
        );
        assert_eq!(
            crate::Literal::fixed_parser().parse(".3d"),
            Ok(crate::Literal::FixedPoint(0, 3))
        );
        assert_eq!(
            crate::Literal::fixed_parser().parse("3d"),
            Ok(crate::Literal::FixedPoint(3, 0))
        );
    }

    #[test]
    fn parse_literal() {
        use crate::Literal;
        let p = crate::Literal::parser();

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
