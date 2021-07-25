use std::{
    ops::Index,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::model::{
    FromParticipant, InteractionMessage, Line, LineContents, MetaDataType, ToParticipant,
};
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use Ordering::Relaxed;

lazy_static! {
    static ref INTERACTION_REGEX: Regex = Regex::new("^(.+)\\s+-+>+\\s+([^:]+):?(.*)$").unwrap();
}

// == Document Parser =====================================
pub struct DocumentParser;
impl DocumentParser {
    pub fn parse(line: &str) -> Vec<Line> {
        let atomic_line_number = AtomicU32::new(0);
        line.lines()
            .into_iter()
            .map(|line| line.trim())
            .map(|line| {
                let line_number = atomic_line_number.fetch_add(1, Relaxed);
                let line_data = line.to_owned();
                let line_contents = if line.is_empty() {
                    LineContents::Empty
                } else if line.starts_with('#') {
                    LineContents::Comment
                } else if line.starts_with(':') {
                    DocumentParser::parse_metadata(&line)
                } else if line.contains("->") {
                    DocumentParser::parse_interaction(&line)
                } else {
                    LineContents::Invalid
                };
                Line {
                    line_number,
                    line_data,
                    line_contents,
                }
            })
            .collect_vec()
    }

    #[inline(always)]
    fn parse_interaction(line: &str) -> LineContents {
        match INTERACTION_REGEX.captures(line) {
            None => LineContents::Invalid,
            Some(captures) => {
                let from_name = FromParticipant(captures.index(1).trim().to_owned());
                let to_name = ToParticipant(captures.index(2).trim().to_owned());
                if captures.len() >= 3 && !captures.index(3).is_empty() {
                    let msg = InteractionMessage(captures.index(3).trim().to_owned());
                    LineContents::InteractionWithMessage(from_name, to_name, msg)
                } else {
                    LineContents::Interaction(from_name, to_name)
                }
            }
        }
    }

    #[inline(always)]
    fn parse_metadata(line: &str) -> LineContents {
        if let Some(o) = line.trim().split_once(|c: char| c.is_whitespace()) {
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

#[test]
fn test_document_parser_with_invalid() {
    let text = "    Client -> Server: Message
    Server
    -> Server: Response";
    let vec = DocumentParser::parse(text);
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
    let vec = DocumentParser::parse(text);
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
