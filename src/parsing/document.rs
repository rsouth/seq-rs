use std::sync::OnceLock;

use crate::model::{
    Config, FromParticipant, InteractionMessage, Line, LineContents, MetaDataType, ToParticipant,
};
use itertools::Itertools;
use regex::Regex;

static INTERACTION_REGEX: OnceLock<Regex> = OnceLock::new();

fn interaction_regex() -> &'static Regex {
    INTERACTION_REGEX.get_or_init(|| Regex::new(r"^(.+)\s+-+>+\s+([^:]+):?(.*)$").unwrap())
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
    pub fn parse(input: &[String], config: Config) -> Document {
        let lines = input
            .iter()
            .enumerate()
            .map(|(line_number, line)| {
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
                Line {
                    line_number,
                    line_data,
                    line_contents,
                }
            })
            .collect_vec();

        Document {
            lines,
            config,
            is_valid: true,
        }
    }

    #[inline]
    fn parse_interaction(line: &str) -> LineContents {
        let line = line.trim();
        match interaction_regex().captures(line) {
            None => LineContents::Invalid,
            Some(captures) => {
                let from_name = FromParticipant(captures[1].trim().to_owned());
                let to_name = ToParticipant(captures[2].trim().to_owned());
                if captures.len() >= 3 && !captures[3].is_empty() {
                    let msg = InteractionMessage(captures[3].trim_start().to_owned());
                    LineContents::InteractionWithMessage(from_name, to_name, msg)
                } else {
                    LineContents::Interaction(from_name, to_name)
                }
            }
        }
    }

    #[inline]
    fn parse_metadata(line: &str) -> LineContents {
        if let Some((key, value)) = line.trim().split_once(|c: char| c.is_whitespace()) {
            let meta = match key {
                ":theme" => MetaDataType::Style(value.trim().to_owned()),
                ":title" => MetaDataType::Title(value.trim().to_owned()),
                ":author" => MetaDataType::Author(value.trim().to_owned()),
                ":date" => MetaDataType::Date,
                _ => MetaDataType::Invalid,
            };
            LineContents::MetaData(meta)
        } else {
            LineContents::Invalid
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn str_to_vec(s: &str) -> Vec<String> {
        s.lines().map(|p| p.to_string()).collect_vec()
    }
    use crate::model::Source;

    fn make_config() -> Config {
        Config {
            input_source: Source::Example,
            output_path: String::new(),
        }
    }

    #[test]
    fn test_parse_metadata_title() {
        assert_eq!(
            LineContents::MetaData(MetaDataType::Title("Test".to_string())),
            DocumentParser::parse_metadata(":title Test")
        );
    }

    #[test]
    fn test_parse_metadata_title_with_whitespace() {
        assert_eq!(
            LineContents::MetaData(MetaDataType::Title("Test".to_string())),
            DocumentParser::parse_metadata("  :title   Test  ")
        );
    }

    #[test]
    fn test_parse_metadata_theme() {
        assert_eq!(
            LineContents::MetaData(MetaDataType::Style("Dark".to_string())),
            DocumentParser::parse_metadata(":theme Dark")
        );
    }

    #[test]
    fn test_parse_metadata_author() {
        assert_eq!(
            LineContents::MetaData(MetaDataType::Author("Alice".to_string())),
            DocumentParser::parse_metadata(":author Alice")
        );
    }

    #[test]
    fn test_parse_metadata_date() {
        assert_eq!(
            LineContents::MetaData(MetaDataType::Date),
            DocumentParser::parse_metadata(":date today")
        );
    }

    #[test]
    fn test_parse_metadata_unknown_key() {
        assert_eq!(
            LineContents::MetaData(MetaDataType::Invalid),
            DocumentParser::parse_metadata(":unknown value")
        );
    }

    #[test]
    fn test_parse_metadata_no_value_returns_invalid() {
        // A bare `:title` with no whitespace after → split_once fails → Invalid
        assert_eq!(LineContents::Invalid, DocumentParser::parse_metadata(":title"));
    }

    #[test]
    fn test_parse_interaction_simple() {
        assert_eq!(
            LineContents::Interaction(
                FromParticipant("Client".to_string()),
                ToParticipant("Server".to_string())
            ),
            DocumentParser::parse_interaction("Client -> Server")
        );
    }

    #[test]
    fn test_parse_interaction_with_message() {
        assert_eq!(
            LineContents::InteractionWithMessage(
                FromParticipant("Client".to_string()),
                ToParticipant("Server".to_string()),
                InteractionMessage("Hello".to_string())
            ),
            DocumentParser::parse_interaction("Client -> Server: Hello")
        );
    }

    #[test]
    fn test_parse_interaction_no_match() {
        assert_eq!(
            LineContents::Invalid,
            DocumentParser::parse_interaction("not an interaction")
        );
    }

    #[test]
    fn test_document_parser_with_invalid() {
        let text = "    Client -> Server: Message
    Server
    -> Server: Response";
        let sss = str_to_vec(text);
        let doc = DocumentParser::parse(&sss, make_config());
        assert_eq!(3, doc.lines.len());

        assert_eq!(0, doc.lines[0].line_number);
        assert_eq!(
            LineContents::InteractionWithMessage(
                FromParticipant("Client".to_string()),
                ToParticipant("Server".to_string()),
                InteractionMessage("Message".to_string())
            ),
            doc.lines[0].line_contents
        );

        assert_eq!(1, doc.lines[1].line_number);
        assert_eq!(LineContents::Invalid, doc.lines[1].line_contents);

        assert_eq!(2, doc.lines[2].line_number);
        assert_eq!(LineContents::Invalid, doc.lines[2].line_contents);
    }

    #[test]
    fn test_document_parser() {
        let text = "
    :title Test
    Client -> Server: Message
    Server -> Database
    Database -> Server: Response";
        let sss = str_to_vec(text);
        let doc = DocumentParser::parse(&sss, make_config());
        assert_eq!(5, doc.lines.len());

        assert_eq!(0, doc.lines[0].line_number);
        assert_eq!(LineContents::Empty, doc.lines[0].line_contents);
        assert_eq!("", doc.lines[0].line_data);

        assert_eq!(1, doc.lines[1].line_number);
        assert_eq!(
            LineContents::MetaData(MetaDataType::Title("Test".to_string())),
            doc.lines[1].line_contents
        );
        assert_eq!(":title Test", doc.lines[1].line_data);

        assert_eq!(2, doc.lines[2].line_number);
        assert_eq!(
            LineContents::InteractionWithMessage(
                FromParticipant("Client".to_string()),
                ToParticipant("Server".to_string()),
                InteractionMessage("Message".to_string())
            ),
            doc.lines[2].line_contents
        );
        assert_eq!("Client -> Server: Message", doc.lines[2].line_data);

        assert_eq!(3, doc.lines[3].line_number);
        assert_eq!(
            LineContents::Interaction(
                FromParticipant("Server".to_string()),
                ToParticipant("Database".to_string())
            ),
            doc.lines[3].line_contents
        );
        assert_eq!("Server -> Database", doc.lines[3].line_data);

        assert_eq!(4, doc.lines[4].line_number);
        assert_eq!(
            LineContents::InteractionWithMessage(
                FromParticipant("Database".to_string()),
                ToParticipant("Server".to_string()),
                InteractionMessage("Response".to_string())
            ),
            doc.lines[4].line_contents
        );
        assert_eq!("Database -> Server: Response", doc.lines[4].line_data);
    }

    #[test]
    fn test_document_parser_comment_lines() {
        let text = "# this is a comment\nClient -> Server";
        let doc = DocumentParser::parse(&str_to_vec(text), make_config());
        assert_eq!(2, doc.lines.len());
        assert_eq!(LineContents::Comment, doc.lines[0].line_contents);
        assert_eq!(
            LineContents::Interaction(
                FromParticipant("Client".to_string()),
                ToParticipant("Server".to_string())
            ),
            doc.lines[1].line_contents
        );
    }

    #[test]
    fn test_document_is_valid() {
        let doc = DocumentParser::parse(&str_to_vec("Client -> Server"), make_config());
        assert!(doc.is_valid);
    }
}
