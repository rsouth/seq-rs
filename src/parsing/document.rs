use std::ops::Index;

use crate::model::{
    FromParticipant, InteractionMessage, Line, LineContents, MetaDataType, ToParticipant,
};
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use std::ops::AddAssign;

lazy_static! {
    static ref INTERACTION_REGEX: Regex = Regex::new("^(.+)\\s+-+>+\\s+([^:]+):?(.*)$").unwrap();
}

// == Document Parser =====================================
pub struct DocumentParser;
impl DocumentParser {
    pub fn parse(line: Vec<String>) -> Vec<Line> {
        let mut atomic_line_number: usize = 0;
        line.iter()
            .map(|line| line.trim())
            .map(|line| {
                atomic_line_number.add_assign(1);
                let line_data = line.to_owned();
                let line_contents = if line.is_empty() {
                    LineContents::Empty
                } else if line.starts_with('#') {
                    LineContents::Comment
                } else if line.starts_with(':') {
                    DocumentParser::parse_metadata(line.trim())
                } else if line.contains("->") {
                    DocumentParser::parse_interaction(line.trim())
                } else {
                    LineContents::Invalid
                };
                Line {
                    line_number: atomic_line_number,
                    line_data,
                    line_contents,
                }
            })
            .collect_vec()
    }

    #[inline(always)]
    fn parse_interaction(line: &str) -> LineContents {
        let line = line.to_owned();
        match INTERACTION_REGEX.captures(&line) {
            None => LineContents::Invalid,
            Some(captures) => {
                let from_name = FromParticipant(captures.index(1).to_owned());
                let to_name = ToParticipant(captures.index(2).to_owned());
                if captures.len() >= 3 && !captures.index(3).is_empty() {
                    let msg = InteractionMessage(captures.index(3).trim_start().to_owned());
                    LineContents::InteractionWithMessage(from_name, to_name, msg)
                } else {
                    LineContents::Interaction(from_name, to_name)
                }
            }
        }
    }

    #[inline(always)]
    fn parse_metadata(line: &str) -> LineContents {
        if let Some(o) = line.split_once(|c: char| c.is_whitespace()) {
            let x = match o.0 {
                ":theme" => MetaDataType::Style(o.1.to_owned()),
                ":title" => MetaDataType::Title(o.1.to_owned()),
                ":author" => MetaDataType::Author(o.1.to_owned()),
                ":date" => MetaDataType::Date,
                &_ => MetaDataType::Invalid,
            };
            LineContents::MetaData(x)
        } else {
            LineContents::Invalid
        }
    }
}

fn _str_to_vec(s: &str) -> Vec<String> {
    s.lines().into_iter().map(|p| p.to_string()).collect_vec()
}

#[test]
fn test_document_parser_with_invalid() {
    let text = "    Client -> Server: Message
    Server
    -> Server: Response";
    let sss = str_to_vec(text);
    let vec = DocumentParser::parse(sss);
    assert_eq!(3, vec.len());

    // line 0
    let client_participant = FromParticipant("Client".to_string());
    let server_participant = ToParticipant("Server".to_string());
    let expect_msg = InteractionMessage("Message".to_string());
    assert_eq!(0, vec[0].line_number);
    assert_eq!(
        LineContents::InteractionWithMessage(client_participant, server_participant, expect_msg),
        vec[0].line_contents
    );

    // line 1
    assert_eq!(1, vec[1].line_number);
    assert_eq!(LineContents::Invalid, vec[1].line_contents);

    // line 2
    assert_eq!(2, vec[2].line_number);
    assert_eq!(LineContents::Invalid, vec[2].line_contents);
}

#[test]
fn test_document_parser() {
    let text = "
    :title Test
    Client -> Server: Message
    Server -> Database
    Database -> Server: Response";
    let sss = str_to_vec(text);
    let vec = DocumentParser::parse(sss);
    assert_eq!(5, vec.len());

    // line 0
    assert_eq!(0, vec[0].line_number);
    assert_eq!(LineContents::Empty, vec[0].line_contents);
    assert_eq!("", vec[0].line_data);

    // line 1
    assert_eq!(1, vec[1].line_number);
    assert_eq!(
        LineContents::MetaData(MetaDataType::Title("Test".to_string())),
        vec[1].line_contents
    );
    assert_eq!(":title Test", vec[1].line_data);

    // line 2
    let expect_from = FromParticipant("Client".to_string());
    let expect_to = ToParticipant("Server".to_string());
    let expect_msg = InteractionMessage("Message".to_string());
    assert_eq!(2, vec[2].line_number);
    assert_eq!(
        LineContents::InteractionWithMessage(expect_from, expect_to, expect_msg),
        vec[2].line_contents
    );
    assert_eq!("Client -> Server: Message", vec[2].line_data);

    // line 3
    let expect_from = FromParticipant("Server".to_string());
    let expect_to = ToParticipant("Database".to_string());
    assert_eq!(3, vec[3].line_number);
    assert_eq!(
        LineContents::Interaction(expect_from, expect_to),
        vec[3].line_contents
    );
    assert_eq!("Server -> Database", vec[3].line_data);

    // line 4
    let expect_from = FromParticipant("Database".to_string());
    let expect_to = ToParticipant("Server".to_string());
    let expect_msg = InteractionMessage("Response".to_string());
    assert_eq!(4, vec[4].line_number);
    assert_eq!(
        LineContents::InteractionWithMessage(expect_from, expect_to, expect_msg),
        vec[4].line_contents
    );
    assert_eq!("Database -> Server: Response", vec[4].line_data);
}
