use crate::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Element {
    name: String,
    attributes: Vec<(String, String)>,
    children: Vec<Element>,
}

/// an xml identifier
fn identifier(input: &str) -> ParseResult<String> {
    let mut matched = String::new();
    let mut chars = input.chars();

    match chars.next() {
        Some(next) if next.is_alphabetic() => matched.push(next),
        _ => return Err(input)
    }

    while let Some(next) = chars.next() {
        if next.is_alphanumeric() || next == '-' {
            matched.push(next);
        } else {
            break;
        }
    }

    let next_index = matched.len();
    Ok((&input[next_index..], matched))
}

/// an xml element
pub fn element<'a>() -> impl Parser<'a, Element> {
    whitespace_wrap(either(single_element(), parent_element()))
}

/// start of an xml element. i.e. <test
fn element_start<'a>() -> impl Parser<'a, (String, Vec<(String, String)>)> {
    right(match_literal("<"), pair(identifier, attributes()))
}

/// opening of an xml element. i.e. <test>
fn open_element<'a>() -> impl Parser<'a, Element> {
    left(element_start(), match_literal(">")).map(|(name, attributes)| Element {
        name,
        attributes,
        children: vec![],
    })
}

/// closing of an xml element. i.e. </test>
fn close_element<'a>(expected_name: String) -> impl Parser<'a, String> {
    right(match_literal("</"), left(identifier, match_literal(">")))
        .pred(move |name| name == &expected_name)
}

/// parent element with children
fn parent_element<'a>() -> impl Parser<'a, Element> {
    open_element().and_then(|el| {
        left(zero_or_more(element()), close_element(el.name.clone())).map(move |children| {
            let mut el = el.clone();
            el.children = children;
            el
        })
    })
}

/// attribute pair like name="value"
fn attribute_pair<'a>() -> impl Parser<'a, (String, String)> {
    pair(identifier, right(match_literal("="), quoted_string()))
}

/// attributes list
fn attributes<'a>() -> impl Parser<'a, Vec<(String, String)>> {
    zero_or_more(right(space1(), attribute_pair()))
}

/// single xml element. i.e. <test />
pub fn single_element<'a>() -> impl Parser<'a, Element> {
    left(element_start(), match_literal("/>")).map(
        |(name, attributes)| Element {
            name,
            attributes,
            children: vec![],
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifier_parser() {
        assert_eq!(Ok(("", "i-am-identifier".to_owned())), identifier("i-am-identifier"));
        assert_eq!(Ok((" all identifier", "not".to_owned())), identifier("not all identifier"));
        assert_eq!(Err("!not an identifier"), identifier("!not an identifier"));
        assert_eq!(Err("!not"), identifier("!not"));
    }


    #[test]
    fn single_element_parser() {
        assert_eq!(
            Ok((
                "",
                Element {
                    name: "div".to_string(),
                    attributes: vec![("class".to_string(), "float".to_string())],
                    children: vec![],
                }
            )),
            single_element().parse("<div class=\"float\"/>")
        );
    }

    #[test]
    fn attribute_parser() {
        assert_eq!(
            Ok((
                "",
                vec![
                    ("one".to_string(), "1".to_string()),
                    ("two".to_string(), "2".to_string())
                ]
            )),
            attributes().parse(" one=\"1\" two=\"2\"")
        );
    }

    #[test]
    fn xml_parser() {
        let doc = r#"
        <top label="Top">
            <semi-bottom label="Bottom"/>
            <middle>
                <bottom label="Another bottom"/>
            </middle>
        </top>"#;
        let parsed_doc = Element {
            name: "top".to_string(),
            attributes: vec![("label".to_string(), "Top".to_string())],
            children: vec![
                Element {
                    name: "semi-bottom".to_string(),
                    attributes: vec![("label".to_string(), "Bottom".to_string())],
                    children: vec![],
                },
                Element {
                    name: "middle".to_string(),
                    attributes: vec![],
                    children: vec![Element {
                        name: "bottom".to_string(),
                        attributes: vec![("label".to_string(), "Another bottom".to_string())],
                        children: vec![],
                    }],
                },
            ],
        };
        assert_eq!(Ok(("", parsed_doc)), element().parse(doc));
    }

    #[test]
    fn mismatched_closing_tag() {
        let doc = r#"
        <top>
            <bottom/>
        </middle>"#;
        assert_eq!(Err("</middle>"), element().parse(doc));
    }
}