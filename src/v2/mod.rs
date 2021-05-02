use crate::rendering::render_context::RenderingContext;
use core::fmt;
use std::error::Error;
use std::str::Lines;

pub mod parsing;

pub type ParseResult<T> = std::result::Result<T, ParseError>;
pub type DrawResult = std::result::Result<(), DrawError>;
pub type InteractionSet = Vec<Interaction>;
pub type ParticipantSet = Vec<Participant>;

// == Diagram =============================================
#[derive(Debug)]
pub struct Diagram {
    pub interactions: InteractionSet,
    pub participants: ParticipantSet,
}

// == Participant =========================================
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub struct Participant {
    pub name: String,
}

// == Interaction =========================================
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub struct Interaction {
    pub from_participant: Participant,
    pub to_participant: Participant,
    pub message: Option<Message>,
}

// == Message =============================================
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub struct Message(pub String);

// == Parse Error =========================================
#[derive(Debug, Clone)]
pub enum ParseError {
    InteractionParseFail,
}

impl Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::InteractionParseFail => write!(f, "Failed to parser Interaction"),
        }
    }
}

// == Draw Error ==========================================
#[derive(Debug, Clone)]
pub enum DrawError {
    DrawFail,
}

impl Error for DrawError {}

impl fmt::Display for DrawError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DrawError::DrawFail => write!(f, "Failed to draw something"),
        }
    }
}

// == Parse Trait =========================================
pub trait Parse<T> {
    fn parse(lines: Lines) -> Result<T, ParseError>;
}

// == Draw Trait ==========================================
pub trait Draw {
    fn draw(&self, rc: &mut RenderingContext) -> DrawResult;
}

// == Tests ===============================================
#[test]
fn test() {
    let input = "Client -> Server".lines();
    let result = InteractionSet::parse(input).unwrap();
    assert_eq!(1, result.len());
    assert_eq!("Client", result.first().unwrap().from_participant.name);
    assert_eq!("Server", result.first().unwrap().to_participant.name);
    assert_eq!(None, result.first().unwrap().message);

    let input = "Client -> Server: Message\nServer -> Database: Response".lines();
    let result = InteractionSet::parse(input).unwrap();
    assert_eq!(2, result.len());
    assert_eq!("Client", result.first().unwrap().from_participant.name);
    assert_eq!("Server", result.first().unwrap().to_participant.name);
    assert_eq!(
        "Message",
        result.first().unwrap().message.as_ref().unwrap().0
    );
    assert_eq!("Server", result.get(1).unwrap().from_participant.name);
    assert_eq!("Database", result.get(1).unwrap().to_participant.name);
    assert_eq!(
        "Response",
        result.get(1).unwrap().message.as_ref().unwrap().0
    );
}

#[test]
fn test_diagram() {
    let input = "
    Client -> Server : Request
    Server -> Database: Query
    Database -> Server
    Server --> Client: Response
    ";
    let diagram = Diagram::parse(input.lines()).unwrap();
    assert_eq!(4, diagram.interactions.len());
    assert_eq!(
        "Client",
        diagram.interactions.first().unwrap().from_participant.name
    );
    assert_eq!(
        "Server",
        diagram.interactions.first().unwrap().to_participant.name
    );
    assert_eq!(
        "Request",
        diagram
            .interactions
            .first()
            .unwrap()
            .message
            .as_ref()
            .unwrap()
            .0
    );

    assert_eq!(3, diagram.participants.len());
}
