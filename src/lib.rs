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

    pub fn parser() -> impl Parser<char, Literal, Error = Simple<char>> {
        Self::bool_parser().or(Self::int_parser())
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
}
