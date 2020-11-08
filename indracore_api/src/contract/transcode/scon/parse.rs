// Copyright 2018-2020 Parity Technologies (UK) Ltd.
// This file is part of cargo-contract.
//
// cargo-contract is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// cargo-contract is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with cargo-contract.  If not, see <http://www.gnu.org/licenses/>.

use super::{Bytes, Map, Tuple, Value};
use escape8259::unescape;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{alphanumeric1, anychar, char, digit0, multispace0, one_of},
    combinator::{map, map_res, opt, recognize, value, verify},
    error::{ErrorKind, ParseError},
    multi::{many0, many0_count, separated_list},
    sequence::{delimited, pair, preceded, separated_pair, tuple},
    IResult,
};
use std::num::ParseIntError;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum SonParseError {
    #[error("bad integer")]
    BadInt(#[from] ParseIntError),
    #[error("bad escape sequence")]
    BadEscape,
    #[error("hex string parse error")]
    BadHex(#[from] hex::FromHexError),
    #[error("unknown parser error")]
    Unparseable,
}

impl<I> ParseError<I> for SonParseError {
    fn from_error_kind(_input: I, _kind: ErrorKind) -> Self {
        SonParseError::Unparseable
    }

    fn append(_: I, _: ErrorKind, other: Self) -> Self {
        other
    }
}

fn scon_string(input: &str) -> IResult<&str, Value, SonParseError> {
    // There are only two types of escape allowed by RFC 8259.
    // - single-character escapes \" \\ \/ \b \f \n \r \t
    // - general-purpose \uXXXX
    // Note: we don't enforce that escape codes are valid here.
    // There must be a decoder later on.
    fn escape_code(input: &str) -> IResult<&str, &str, SonParseError> {
        recognize(pair(
            tag("\\"),
            alt((
                tag("\""),
                tag("\\"),
                tag("/"),
                tag("b"),
                tag("f"),
                tag("n"),
                tag("r"),
                tag("t"),
                tag("u"),
            )),
        ))(input)
    }

    // Zero or more text characters
    fn string_body(input: &str) -> IResult<&str, &str, SonParseError> {
        recognize(many0(alt((nonescaped_string, escape_code))))(input)
    }

    fn string_literal(input: &str) -> IResult<&str, String, SonParseError> {
        let (remain, raw_string) = delimited(tag("\""), string_body, tag("\""))(input)?;

        match unescape(raw_string) {
            Ok(s) => Ok((remain, s)),
            Err(_) => Err(nom::Err::Failure(SonParseError::BadEscape)),
        }
    }

    map(string_literal, |s| Value::String(s))(input)
}

// A character that is:
// NOT a control character (0x00 - 0x1F)
// NOT a quote character (0x22)
// NOT a backslash character (0x5C)
// Is within the unicode range (< 0x10FFFF) (this is already guaranteed by Rust char)
fn is_nonescaped_string_char(c: char) -> bool {
    let cv = c as u32;
    (cv >= 0x20) && (cv != 0x22) && (cv != 0x5C)
}

// One or more unescaped text characters
fn nonescaped_string(input: &str) -> IResult<&str, &str, SonParseError> {
    take_while1(is_nonescaped_string_char)(input)
}

fn rust_ident(input: &str) -> IResult<&str, &str, SonParseError> {
    recognize(pair(
        verify(anychar, |&c| c.is_alphabetic() || c == '_'),
        many0_count(preceded(opt(char('_')), alphanumeric1)),
    ))(input)
}

fn digit1to9(input: &str) -> IResult<&str, char, SonParseError> {
    one_of("123456789")(input)
}

// unsigned_integer = zero / ( digit1-9 *DIGIT )
fn uint(input: &str) -> IResult<&str, &str, SonParseError> {
    alt((tag("0"), recognize(pair(digit1to9, digit0))))(input)
}

fn scon_integer(input: &str) -> IResult<&str, Value, SonParseError> {
    let signed = recognize(pair(char('-'), uint));

    alt((
        map_res(signed, |s| {
            s.parse::<i128>()
                .map_err(SonParseError::BadInt)
                .map(Value::Int)
        }),
        map_res(uint, |s| {
            s.parse::<u128>()
                .map_err(SonParseError::BadInt)
                .map(Value::UInt)
        }),
    ))(input)
}

fn scon_unit(input: &str) -> IResult<&str, Value, SonParseError> {
    let (i, _) = tag("()")(input)?;
    Ok((i, Value::Unit))
}

fn scon_bool(input: &str) -> IResult<&str, Value, SonParseError> {
    alt((
        value(Value::Bool(false), tag("false")),
        value(Value::Bool(true), tag("true")),
    ))(input)
}

fn scon_char(input: &str) -> IResult<&str, Value, SonParseError> {
    let parse_char = delimited(tag("'"), anychar, tag("'"));
    map(parse_char, |c| Value::Char(c))(input)
}

fn scon_seq(input: &str) -> IResult<&str, Value, SonParseError> {
    let opt_trailing_comma_close = pair(opt(ws(tag(","))), ws(tag("]")));

    let parser = delimited(
        ws(tag("[")),
        separated_list(ws(tag(",")), scon_value),
        opt_trailing_comma_close,
    );
    map(parser, |v| Value::Seq(v.into()))(input)
}

fn scon_tuple(input: &str) -> IResult<&str, Value, SonParseError> {
    let opt_trailing_comma_close = pair(opt(ws(tag(","))), ws(tag(")")));
    let tuple_body = delimited(
        ws(tag("(")),
        separated_list(ws(tag(",")), scon_value),
        opt_trailing_comma_close,
    );

    let parser = tuple((opt(ws(rust_ident)), tuple_body));

    map(parser, |(ident, v)| {
        Value::Tuple(Tuple::new(ident, v.into_iter().collect()))
    })(input)
}

/// Parse a rust ident on its own which could represent a struct with no fields or a enum unit
/// variant e.g. "None"
fn scon_unit_tuple(input: &str) -> IResult<&str, Value, SonParseError> {
    map(rust_ident, |ident| {
        Value::Tuple(Tuple::new(Some(ident), Vec::new()))
    })(input)
}

fn scon_map(input: &str) -> IResult<&str, Value, SonParseError> {
    let ident_key = map(rust_ident, |s| Value::String(s.into()));
    let scon_map_key = ws(alt((ident_key, scon_string, scon_integer)));

    let opening = alt((tag("("), tag("{")));
    let closing = alt((tag(")"), tag("}")));
    let entry = separated_pair(scon_map_key, ws(tag(":")), scon_value);

    let opt_trailing_comma_close = pair(opt(ws(tag(","))), ws(closing));
    let map_body = delimited(
        ws(opening),
        separated_list(ws(tag(",")), entry),
        opt_trailing_comma_close,
    );

    let parser = tuple((opt(ws(rust_ident)), map_body));

    map(parser, |(ident, v)| {
        Value::Map(Map::new(ident, v.into_iter().collect()))
    })(input)
}

fn scon_bytes(input: &str) -> IResult<&str, Value, SonParseError> {
    let (rest, byte_str) = preceded(tag("0x"), nom::character::complete::hex_digit1)(input)?;
    let bytes = Bytes::from_hex_string(byte_str).map_err(|e| nom::Err::Failure(e.into()))?;
    Ok((rest, Value::Bytes(bytes)))
}

fn ws<F, I, O, E>(f: F) -> impl Fn(I) -> IResult<I, O, E>
where
    F: Fn(I) -> IResult<I, O, E>,
    I: nom::InputTakeAtPosition,
    <I as nom::InputTakeAtPosition>::Item: nom::AsChar + Clone,
    E: nom::error::ParseError<I>,
{
    delimited(multispace0, f, multispace0)
}

fn scon_value(input: &str) -> IResult<&str, Value, SonParseError> {
    ws(alt((
        scon_unit,
        scon_bytes,
        scon_seq,
        scon_tuple,
        scon_map,
        scon_string,
        scon_integer,
        scon_bool,
        scon_char,
        scon_unit_tuple,
    )))(input)
}

/// Attempt to parse a SON value
pub fn parse_value(input: &str) -> Result<Value, nom::Err<SonParseError>> {
    let (_, value) = scon_value(input)?;
    Ok(value)
}
