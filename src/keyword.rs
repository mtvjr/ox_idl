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

use std::fmt::Display;
use strum::EnumIter;

/// The Keyword enum lists all the keywords of the
/// IDL language and provides means of iterating
/// and generating parsers for the keywords
#[derive(Debug, Clone, PartialEq, EnumIter)]
pub enum Keyword {
    Abstract,
    Any,
    Alias,
    Attribute,
    Bitfield,
    Bitmask,
    Bitset,
    Boolean,
    Case,
    Char,
    Component,
    Connector,
    Const,
    Consumes,
    Context,
    Custom,
    Default,
    Double,
    Exception,
    Emits,
    Enum,
    EventType,
    Factory,
    False,
    Finder,
    Fixed,
    Float,
    GetRaises,
    Home,
    Import,
    In,
    InOut,
    Interface,
    Local,
    Long,
    Manages,
    Map,
    MirrorPort,
    Module,
    Multiple,
    Native,
    Object,
    Octet,
    OneWay,
    Out,
    PrimaryKey,
    Private,
    Port,
    PortType,
    Provides,
    Public,
    Publishes,
    Raises,
    ReadOnly,
    SetRaises,
    Sequence,
    Short,
    String,
    Struct,
    Supports,
    Switch,
    True,
    Truncatable,
    Typedef,
    TypeId,
    TypeName,
    TypePrefix,
    Unsigned,
    Union,
    Uses,
    ValueBase,
    ValueType,
    Void,
    WChar,
    WString,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
}

impl Keyword {
    /// Builds a parser for a keyword that accepts the IDL defined keyword
    /// and returns the value of the keyword
    ///
    /// Example:
    ///
    /// ```
    /// use ridl::keyword::Keyword;
    /// use chumsky::prelude::*;
    ///
    /// let false_parser = Keyword::False.make_parser();
    ///
    /// let result = false_parser.parse("FALSE");
    /// assert_eq!(result, Ok(Keyword::False));
    /// ```
    pub fn make_parser(&self) -> impl Parser<char, Keyword, Error = Simple<char>> {
        text::keyword(self.to_string()).to(self.clone())
    }
}

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // The intent with this functions is to retrieve the keyword as it
        // is specified in 7.2.4 keywords with case sensitivity
        let s = match &self {
            // These variants have upper case letters
            Keyword::False => "FALSE".to_string(),
            Keyword::Object => "Object".to_string(),
            Keyword::True => "TRUE".to_string(),
            Keyword::ValueBase => "ValueBase".to_string(),
            // All others are lower case only
            _ => format!("{:?}", &self).to_ascii_lowercase(),
        };
        f.write_str(s.as_str())
    }
}

#[cfg(test)]
mod keyword_tests {
    use crate::keyword::Keyword;
    use chumsky::Parser;
    use strum::IntoEnumIterator;

    #[test]
    fn display() {
        assert_eq!(format!("{}", Keyword::Struct), "struct".to_string());
        assert_eq!(Keyword::Struct.to_string(), "struct".to_string());
        assert_eq!(Keyword::True.to_string(), "TRUE".to_string());
    }

    #[test]
    fn iter() {
        assert!(Keyword::iter().find(|k| k == &Keyword::Boolean).is_some());
        assert!(Keyword::iter().find(|k| k == &Keyword::Struct).is_some());
    }

    #[test]
    fn make_parser() {
        assert_eq!(
            Keyword::False.make_parser().parse("FALSE"),
            Ok(Keyword::False)
        );
        assert!(Keyword::False.make_parser().parse("fAlse").is_err());
        assert_eq!(
            Keyword::Struct.make_parser().parse("struct"),
            Ok(Keyword::Struct)
        );
        assert!(Keyword::Struct.make_parser().parse("Struct").is_err());
    }
}
