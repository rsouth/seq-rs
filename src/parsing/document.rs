use std::ops::Index;

use crate::model::{
    Config, FromParticipant, InteractionMessage, Line, LineContents, MetaDataType, ToParticipant,
};
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use std::ops::AddAssign;

lazy_static! {
    static ref INTERACTION_REGEX: Regex = Regex::new("^(.+)\\s+-+>+\\s+([^:]+):?(.*)$").unwrap();
}

#[derive(Debug)]
pub struct Document {
    pub config: Config,
    pub lines: Vec<Line>,
    pub is_valid: bool,
}

// == Document Parser =====================================
pub struct DocumentParser;
impl DocumentParser {
    pub fn parse(line: &[String], config: Config) -> Document {
        // todo Result<...>
        let mut atomic_line_number: usize = 0;
        let lines = line
            .iter()
            // .map(|line| line.trim())
            .map(|line| {
                let line_data = line.trim().to_owned();
                let line_contents = if line_data.is_empty() {
                    LineContents::Empty
                } else if line_data.starts_with('#') {
                    LineContents::Comment
                } else if line_data.starts_with(':') {
                    DocumentParser::parse_metadata(&line_data)
                } else if line.contains("->") {
                    DocumentParser::parse_interaction(&line_data)
                } else {
                    LineContents::Invalid
                };
                let retval = Line {
                    line_number: atomic_line_number,
                    line_data,
                    line_contents,
                };
                atomic_line_number.add_assign(1);
                retval
            })
            .collect_vec();

        Document {
            lines,
            config,
            is_valid: true,
        }
    }

    #[inline(always)]
    fn parse_interaction(line: &str) -> LineContents {
        let line = line.trim().to_owned();
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
        if let Some(o) = line.trim().split_once(|c: char| c.is_whitespace()) {
            let x = match o.0 {
                ":theme" => MetaDataType::Style(o.1.trim().to_owned()),
                ":title" => MetaDataType::Title(o.1.trim().to_owned()),
                ":author" => MetaDataType::Author(o.1.trim().to_owned()),
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
fn test_parse_metadata() {
    assert_eq!(
        LineContents::MetaData(MetaDataType::Title("Test".to_string())),
        DocumentParser::parse_metadata(":title Test")
    );

    assert_eq!(
        LineContents::MetaData(MetaDataType::Title("Test".to_string())),
        DocumentParser::parse_metadata("  :title   Test  ")
    );
}

#[test]
fn test_document_parser_with_invalid() {
    use crate::model::Source;
    let text = "    Client -> Server: Message
    Server
    -> Server: Response";
    let config = Config {
        input_source: Source::Example,
        output_path: "".to_string(),
    };
    let sss = str_to_vec(text);
    let vec = DocumentParser::parse(&sss, config);
    assert_eq!(3, vec.lines.len());

    // line 0
    let client_participant = FromParticipant("Client".to_string());
    let server_participant = ToParticipant("Server".to_string());
    let expect_msg = InteractionMessage("Message".to_string());
    assert_eq!(0, vec.lines[0].line_number);
    assert_eq!(
        LineContents::InteractionWithMessage(client_participant, server_participant, expect_msg),
        vec.lines[0].line_contents
    );

    // line 1
    assert_eq!(1, vec.lines[1].line_number);
    assert_eq!(LineContents::Invalid, vec.lines[1].line_contents);

    // line 2
    assert_eq!(2, vec.lines[2].line_number);
    assert_eq!(LineContents::Invalid, vec.lines[2].line_contents);
}

#[test]
fn test_document_parser() {
    use crate::model::Source;

    let text = "
    :title Test
    Client -> Server: Message
    Server -> Database
    Database -> Server: Response";
    let config = Config {
        input_source: Source::Example,
        output_path: "".to_string(),
    };
    let sss = str_to_vec(text);
    let vec = DocumentParser::parse(&sss, config);
    assert_eq!(5, vec.lines.len());

    // line 0
    assert_eq!(0, vec.lines[0].line_number);
    assert_eq!(LineContents::Empty, vec.lines[0].line_contents);
    assert_eq!("", vec.lines[0].line_data);

    eprintln!("{:?}", vec.lines[0]);
    eprintln!("{:?}", vec.lines[1]);
    eprintln!("{:?}", vec.lines[2]);

    // line 1
    assert_eq!(1, vec.lines[1].line_number);
    assert_eq!(
        LineContents::MetaData(MetaDataType::Title("Test".to_string())),
        vec.lines[1].line_contents
    );
    assert_eq!(":title Test", vec.lines[1].line_data);

    // line 2
    let expect_from = FromParticipant("Client".to_string());
    let expect_to = ToParticipant("Server".to_string());
    let expect_msg = InteractionMessage("Message".to_string());
    assert_eq!(2, vec.lines[2].line_number);
    assert_eq!(
        LineContents::InteractionWithMessage(expect_from, expect_to, expect_msg),
        vec.lines[2].line_contents
    );
    assert_eq!("Client -> Server: Message", vec.lines[2].line_data);

    // line 3
    let expect_from = FromParticipant("Server".to_string());
    let expect_to = ToParticipant("Database".to_string());
    assert_eq!(3, vec.lines[3].line_number);
    assert_eq!(
        LineContents::Interaction(expect_from, expect_to),
        vec.lines[3].line_contents
    );
    assert_eq!("Server -> Database", vec.lines[3].line_data);

    // line 4
    let expect_from = FromParticipant("Database".to_string());
    let expect_to = ToParticipant("Server".to_string());
    let expect_msg = InteractionMessage("Response".to_string());
    assert_eq!(4, vec.lines[4].line_number);
    assert_eq!(
        LineContents::InteractionWithMessage(expect_from, expect_to, expect_msg),
        vec.lines[4].line_contents
    );
    assert_eq!("Database -> Server: Response", vec.lines[4].line_data);
}

//noinspection RsExternalLinter
fn str_to_vec(s: &str) -> Vec<String> {
    s.lines().into_iter().map(|p| p.to_string()).collect_vec()
}
