use crate::combinators::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Element {
    name: String,
    attributes: Vec<(String, String)>,
    children: Vec<Element>,
}

fn whitespace_char<'a>() -> impl Parser<'a, char> {
    parse_any_char.pred(|c| c.is_whitespace())
}

fn quoted_string<'a>() -> impl Parser<'a, String> {
    right(
        match_literal("\""),
        left(
            zero_or_more(parse_any_char.pred(|c| c != &'"')),
            match_literal("\""),
        ),
    )
    .map(|cs| cs.iter().collect())
}

fn parse_any_char(input: &str) -> ParseResult<char> {
    match input.chars().next() {
        Some(next) => Ok((&input[next.len_utf8()..], next)),
        _ => Err(input),
    }
}

fn parse_identifier(input: &str) -> ParseResult<String> {
    let mut matched = String::new();
    let mut chars = input.chars();

    match chars.next() {
        Some(next) if next.is_alphabetic() => matched.push(next),
        _ => return Err(input),
    }

    while let Some(next) = chars.next() {
        if !(next.is_alphanumeric() || next == '-') {
            break;
        }
        matched.push(next);
    }

    Ok((&input[matched.len()..], matched))
}

fn attribute_pair<'a>() -> impl Parser<'a, (String, String)> {
    pair(parse_identifier, right(match_literal("="), quoted_string()))
}

fn attributes<'a>() -> impl Parser<'a, Vec<(String, String)>> {
    zero_or_more(right(one_or_more(whitespace_char()), attribute_pair()))
}

fn element_start<'a>() -> impl Parser<'a, (String, Vec<(String, String)>)> {
    right(match_literal("<"), pair(parse_identifier, attributes()))
}

fn single_element<'a>() -> impl Parser<'a, Element> {
    left(element_start(), whitespace_wrap(match_literal("/>"))).map(|(name, attributes)| Element {
        name,
        attributes,
        children: vec![],
    })
}

fn open_element<'a>() -> impl Parser<'a, Element> {
    left(element_start(), match_literal(">")).map(|(name, attributes)| Element {
        name,
        attributes,
        children: vec![],
    })
}

fn close_element<'a>(identifier: String) -> impl Parser<'a, String> {
    left(
        right(
            match_literal("</"),
            parse_identifier.pred(move |name| name == &identifier),
        ),
        match_literal(">"),
    )
}

fn parent_element<'a>() -> impl Parser<'a, Element> {
    open_element().and_then(|parent| {
        left(zero_or_more(element()), close_element(parent.name.clone())).map(move |children| {
            let mut parent = parent.clone();
            parent.children = children;
            parent
        })
    })
}

fn whitespace_wrap<'a, P, R>(parser: P) -> impl Parser<'a, R>
where
    P: Parser<'a, R>,
{
    right(
        zero_or_more(whitespace_char()),
        left(parser, zero_or_more(whitespace_char())),
    )
}

pub fn element<'a>() -> impl Parser<'a, Element> {
    whitespace_wrap(either(single_element(), parent_element()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifier_parser() {
        assert_eq!(
            Ok(("", "i-am-an-identifier".to_string())),
            parse_identifier("i-am-an-identifier")
        );
        assert_eq!(
            Ok((" entirely an identifier", "not".to_string())),
            parse_identifier("not entirely an identifier")
        );
        assert_eq!(
            Err("!not at all an identifier"),
            parse_identifier("!not at all an identifier")
        );
    }

    #[test]
    fn right_combinator() {
        let tag_opener = right(match_literal("<"), parse_identifier);
        assert_eq!(
            Ok(("/>", "my-first-element".to_string())),
            tag_opener.parse("<my-first-element/>")
        );
        assert_eq!(Err("oops"), tag_opener.parse("oops"));
        assert_eq!(Err("!oops"), tag_opener.parse("<!oops"));
    }

    #[test]
    fn one_or_more_combinator() {
        let parser = one_or_more(match_literal("ha"));
        assert_eq!(Ok(("", vec![(), (), ()])), parser.parse("hahaha"));
        assert_eq!(Err("ahah"), parser.parse("ahah"));
        assert_eq!(Err(""), parser.parse(""));
    }

    #[test]
    fn zero_or_more_combinator() {
        let parser = zero_or_more(match_literal("ha"));
        assert_eq!(Ok(("", vec![(), (), ()])), parser.parse("hahaha"));
        assert_eq!(Ok(("ahah", vec![])), parser.parse("ahah"));
        assert_eq!(Ok(("", vec![])), parser.parse(""));
    }

    #[test]
    fn predicate_combinator() {
        let parser = pred(parse_any_char, |c| c == &'o');
        assert_eq!(Ok(("mg", 'o')), parser.parse("omg"));
        assert_eq!(Err("lol"), parser.parse("lol"));
    }

    #[test]
    fn quoted_string_parser() {
        let parser = quoted_string();
        assert_eq!(
            Ok(("", "Hello world".to_string())),
            parser.parse("\"Hello world\"")
        );
        assert_eq!(Ok(("", "".to_string())), parser.parse("\"\""));
    }

    #[test]
    fn attributes_parser() {
        let parser = attributes();
        assert_eq!(
            Ok((
                "",
                vec![
                    ("foo".to_string(), "FOO".to_string()),
                    ("bar".to_string(), "BAR".to_string())
                ]
            )),
            parser.parse(r#" foo="FOO" bar="BAR""#)
        )
    }

    #[test]
    fn single_element_parser() {
        let parser = single_element();
        assert_eq!(
            (
                "",
                Element {
                    name: "div".to_string(),
                    attributes: vec![("class".to_string(), "float".to_string())],
                    children: vec![]
                },
            ),
            parser.parse(r#"<div class="float"/>"#).unwrap()
        )
    }

    #[test]
    fn self_closing_element_with_space() {
        let parser = element();
        assert_eq!(
            Ok((
                "",
                Element {
                    name: String::from("crunchy-element"),
                    attributes: vec![],
                    children: vec![]
                }
            )),
            parser.parse(r#"<crunchy-element />"#)
        )
    }

    #[test]
    fn xml_parser() {
        let doc = r#"
            <top label="Top">
                <semi-bottom label="Bottom"/>
                <middle>
                    <bottom label="Another bottom"/>
                </middle>
            </top>
        "#;

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

        assert_eq!(Ok(("", parsed_doc)), element().parse(doc))
    }

    #[test]
    fn mismatched_closing_tag() {
        let doc = r#"
        <top>
            <bottom/>
        </middle>"#;
        assert_eq!(Err("middle>"), element().parse(doc));
    }
}
