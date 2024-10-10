use core::str;
use std::
    str::Utf8Error
;

use anyhow::Result;
use anyhow::anyhow;

enum RespDataType {
    SimpleString,   // Simple Strings (First byte: +)
    SimpleError,    // Simple Errors (First byte: -)
    Integer,        // Integers (First byte: :)
    BulkString,     // Bulk Strings (First byte: $)
    Array,          // Arrays (First byte: *)
    Null,           // Nulls (First byte: _) - RESP3
    Boolean,        // Booleans (First byte: #) - RESP3
    Double,         // Doubles (First byte: ,) - RESP3
    BigNumber,      // Big Numbers (First byte: () - RESP3
    BulkError,      // Bulk Errors (First byte: !) - RESP3
    VerbatimString, // Verbatim Strings (First byte: =) - RESP3
    Map,            // Maps (First byte: %) - RESP3
    Attribute,      // Attributes (First byte: `) - RESP3
    Set,            // Sets (First byte: ~) - RESP3
    Push,           // Pushes (First byte: >) - RESP3
}

pub enum RespToken {
    SimpleString(String),
    SimpleError(String),
    Integer(i64),
    BulkString(String),
    Array(Vec<RespToken>),
    Null,
    Boolean(bool),
    Double(f64),
    BigNumber(String),
    BulkError(String),
    VerbatimString(String),
    Map(Vec<(String, RespToken)>),
    Attribute(String),
    Set(Vec<RespToken>),
    Push(String),
}

#[derive(Debug)]
enum ParseError {
    InvalidToken,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Encountered an unexpected token")
    }
}

pub fn parse_bytes(buffer: &Vec<u8>) -> Result<(), Utf8Error> {
    println!("{:?}", &buffer);
    let parse_string = str::from_utf8(buffer)?;
    println!("{parse_string}");

    let tokens = parse(parse_string);
    let response: String = execute_tokens(tokens);

    return Ok(());
}

fn execute_tokens(tokens: Result<Vec<RespToken>>) -> String {
    todo!()
}

fn parse(s: &str) -> Result<Vec<RespToken>> {
    let (head, tail) = s.split_at(1);

    let data_type = match head {
        "+" => Ok(RespDataType::SimpleString),   // Simple Strings
        "-" => Ok(RespDataType::SimpleError),    // Simple Errors
        ":" => Ok(RespDataType::Integer),        // Integers
        "$" => Ok(RespDataType::BulkString),     // Bulk Strings
        "*" => Ok(RespDataType::Array),          // Arrays
        "_" => Ok(RespDataType::Null),           // Nulls (RESP3)
        "#" => Ok(RespDataType::Boolean),        // Booleans (RESP3)
        "," => Ok(RespDataType::Double),         // Doubles (RESP3)
        "(" => Ok(RespDataType::BigNumber),      // Big Numbers (RESP3)
        "!" => Ok(RespDataType::BulkError),      // Bulk Errors (RESP3)
        "=" => Ok(RespDataType::VerbatimString), // Verbatim Strings (RESP3)
        "%" => Ok(RespDataType::Map),            // Maps (RESP3)
        "`" => Ok(RespDataType::Attribute),      // Attributes (RESP3)
        "~" => Ok(RespDataType::Set),            // Sets (RESP3)
        ">" => Ok(RespDataType::Push),           // Pushes (RESP3)
        _ => Err(anyhow!(ParseError::InvalidToken)),      // Catch-all for invalid tokens
    }?;

    let token = match data_type {
        RespDataType::SimpleString => parse_simple_string(tail),
        RespDataType::SimpleError => parse_simple_error(tail),
        RespDataType::Integer => parse_simple_integer(tail),
        RespDataType::BulkString => parse_bulk_string(tail),
        RespDataType::Array => parse_array(tail),
        RespDataType::Null => parse_null(tail),
        RespDataType::Boolean => parse_boolean(tail),
        RespDataType::Double => parse_double(tail),
        RespDataType::BigNumber => parse_big_number(tail),
        RespDataType::BulkError => parse_bulk_error(tail),
        RespDataType::VerbatimString => parse_verbatim_string(tail),
        RespDataType::Map => parse_map(tail),
        RespDataType::Attribute => parse_attribute(tail),
        RespDataType::Set => parse_set(tail),
        RespDataType::Push => parse_push(tail),
    }?;

    Ok(vec![token])
}

fn parse_simple_string(s: &str) -> Result<RespToken> {
    if let Some(inner_string) = s.strip_suffix("\r\n") {
        return Ok(RespToken::SimpleString(inner_string.to_owned()));
    }
    return Err(anyhow!(ParseError::InvalidToken));
}

fn parse_simple_error(s: &str) -> Result<RespToken> {
    todo!();
}

fn parse_simple_integer(s: &str) -> Result<RespToken> {
    todo!();
}

fn parse_bulk_string(s: &str) -> Result<RespToken> {
    if let None = s.strip_suffix("\r\n"){
        return Err(anyhow!(ParseError::InvalidToken))
    }
    let split_idx = s.find("\r\n").ok_or(anyhow!(ParseError::InvalidToken))?;
    let length = &s[0..split_idx].parse::<usize>()?;
    let carriage_length = 2;
    let inner_string = &s[split_idx+carriage_length..split_idx+carriage_length+length];
    return Ok(RespToken::BulkString(inner_string.to_owned()));
}

fn parse_array(s: &str) -> Result<RespToken> {
    todo!();
}

fn parse_null(s: &str) -> Result<RespToken> {
    todo!();
}

fn parse_boolean(s: &str) -> Result<RespToken> {
    todo!();
}

fn parse_double(s: &str) -> Result<RespToken> {
    todo!();
}

fn parse_big_number(s: &str) -> Result<RespToken> {
    todo!();
}

fn parse_bulk_error(s: &str) -> Result<RespToken> {
    todo!();
}

fn parse_verbatim_string(s: &str) -> Result<RespToken> {
    todo!();
}

fn parse_map(s: &str) -> Result<RespToken> {
    todo!();
}

fn parse_attribute(s: &str) -> Result<RespToken> {
    todo!();
}

fn parse_set(s: &str) -> Result<RespToken> {
    todo!();
}

fn parse_push(s: &str) -> Result<RespToken> {
    todo!();
}

#[cfg(test)]
mod tests {
    use std::iter::zip;

    use super::{parse_bulk_string, RespToken};

    #[test]
    fn test_parse_bulk_string() {
        let cases = vec!["5\r\nhello\r\n", "17\r\nLine1\nLine2\nLine3\r\n"];
        let expected = vec!["hello", "Line1\nLine2\nLine3"];

        for (case, expect) in zip(cases, expected) {
            let res = parse_bulk_string(case).unwrap();
            if let RespToken::BulkString(inner) = res {
                assert_eq!(inner, expect);
            } else {
                panic!("Polymorphisim error")
            }
        }
    }
}