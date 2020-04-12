use std::collections::HashMap;
use std::hash::Hash;

use crate::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Element {
    Null,
    Bool(bool),
    Number(i32),
    String(String),
    Array(Vec<Element>),
    Object(HashMap<String, Element>),
}

fn element(input: &str) -> ParseResult<Element> {
    null(input)
        .map(|(next_input, _)| (next_input, Element::Null))
        .or_else(
            |_| boolean(input).map(|(next_input, boolean_value)| (next_input, Element::Bool(boolean_value)))
        )
        .or_else(
            |_| number(input).map(|(next_input, number_value)| (next_input, Element::Number(number_value)))
        )
        .or_else(
            |_| quoted_string().parse(input).map(|(next_input, string_value)| (next_input, Element::String(string_value)))
        )
}

fn element_pair(input: &str) -> ParseResult<(String, Element)> {
    quoted_string().parse(input).and_then(|(next_input, name)| {
        right(whitespace_wrap(match_literal(":")), element).parse(next_input)
            .map(|(next_input, elem)| (next_input, (name, elem)))
    })
}

fn element_pairs(input: &str) -> ParseResult<Vec<(String, Element)>> {
    one_or_more(
        left(
            element_pair,
            zero_or_more(whitespace_wrap(match_literal(","))),
        )
    ).parse(input)
}

fn object_start(input: &str) -> ParseResult<(String, Element)> {
    right(
        whitespace_wrap(match_literal("{")),
        element_pair,
    ).parse(input)
}

fn object_body(input: &str) -> ParseResult<HashMap<String, Element>> {
    let mut output = HashMap::new();
    object_start(input).and_then(move |(next_input, (name, elem))| {
        output.insert(name, elem);
        zero_or_more(
            pair(whitespace_wrap(match_literal(",")), element_pair)
        )
            .parse(next_input)
            .map(move |(next_input, pairs)| {
                for (ni, (n, e)) in pairs {
                    output.insert(n, e);
                }
                (next_input, output)
            })
    })
}

fn object(input: &str) -> ParseResult<HashMap<String, Element>> {
    left(
        object_body,
        whitespace_wrap(match_literal("}")),
    ).parse(input)
}

fn null(input: &str) -> ParseResult<()> {
    match_literal("null").parse(input)
}

fn boolean(input: &str) -> ParseResult<bool> {
    match match_literal("true").parse(input) {
        Ok((next_input, _)) => Ok((next_input, true)),
        Err(_) => match match_literal("false").parse(input) {
            Ok((next_input, _)) => Ok((next_input, false)),
            Err(err) => Err(err),
        }
    }
}

fn number(input: &str) -> ParseResult<i32> {
    let mut matched = String::new();
    let mut chars = input.chars();

    while let Some(next) = chars.next() {
        if next.is_numeric() {
            matched.push(next);
        } else {
            break;
        }
    }

    let next_index = matched.len();

    matched.parse::<i32>().map(|num| (&input[next_index..], num))
        .map_err(|_| input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_match() {
        assert_eq!(Ok(("", ())), null("null"));
        assert_eq!(Err("nil"), boolean("nil"));
    }

    #[test]
    fn boolean_match() {
        assert_eq!(Ok(("", true)), boolean("true"));
        assert_eq!(Ok(("", false)), boolean("false"));
        assert_eq!(Err("aaa"), boolean("aaa"));
    }

    #[test]
    fn number_match() {
        assert_eq!(Ok(("", 1)), number("1"));
        assert_eq!(Ok(("a", 123)), number("123a"));
        assert_eq!(Err("ddwedq"), number("ddwedq"));
    }

    #[test]
    fn element_match() {
        assert_eq!(Ok(("", Element::Number(1))), element("1"));
        assert_eq!(Ok(("", Element::Null)), element("null"));
        assert_eq!(Ok(("", Element::Bool(false))), element("false"));
        assert_eq!(Ok(("", Element::String("test value".to_owned()))), element("\"test value\""));
        assert_eq!(Err("whatever"), element("whatever"));
    }

    #[test]
    fn element_pair_match() {
        assert_eq!(Ok(("", ("test".to_owned(), Element::Bool(true)))), element_pair("\"test\":true"));
        assert_eq!(Ok(("", ("test".to_owned(), Element::Number(1)))), element_pair("\"test\":1"));
        assert_eq!(Ok(("", ("test".to_owned(), Element::Number(1)))), element_pair("\"test\" : 1"));
        assert_eq!(
            Ok(("", ("test with multiple words".to_owned(), Element::String("value".to_owned())))),
            element_pair("\"test with multiple words\":\"value\"")
        );
        assert_eq!(Err(":1"), element_pair("\"test\"::1"));
    }

    #[test]
    fn elements_pair_match() {
        assert_eq!(
            Ok(("", vec![("test".to_owned(), Element::Bool(true)), ("test2".to_owned(), Element::Bool(false))])),
            element_pairs("\"test\": true, \"test2\": false")
        );

        assert_eq!(
            Ok(("   ", vec![("test".to_owned(), Element::Bool(true)), ("test2".to_owned(), Element::Bool(false))])),
            element_pairs("\"test\" : true , \"test2\" : false   ")
        );

        assert_eq!(
            Ok(("", vec![("test".to_owned(), Element::Bool(true)), ("test2".to_owned(), Element::Bool(false))])),
            element_pairs("\"test\" : true , \"test2\" : false,")
        );
    }

    #[test]
    fn object_match() {
        let mut expected = HashMap::new();
        expected.insert("test".to_owned(), Element::Bool(true));

        assert_eq!(
            Ok(("", expected)),
            object("{\"test\":true}")
        );

        let mut expected = HashMap::new();
        expected.insert("test".to_owned(), Element::Bool(true));
        expected.insert("test2".to_owned(), Element::Number(3));

        assert_eq!(
            Ok(("", expected)),
            object("{ \"test\": true, \"test2\": 3   }")
        )
    }
}