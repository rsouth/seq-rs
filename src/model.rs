use crate::rendering::Rect;

// == Message =============================================
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub struct Message(pub String);

// == Line ================================================
#[derive(Debug)]
pub struct Line {
    pub line_number: usize,
    pub line_contents: LineContents,
    pub line_data: String,
}

// == Line Contents =======================================
#[derive(Debug, PartialOrd, PartialEq)]
pub enum LineContents {
    Invalid,
    Empty,
    Comment,
    MetaData(MetaDataType),
    Interaction(FromParticipant, ToParticipant),
    InteractionWithMessage(FromParticipant, ToParticipant, InteractionMessage),
}

#[derive(Debug, PartialOrd, PartialEq)]
pub enum MetaDataType {
    Style(String), // enum for styles ??
    FontSize(f32),
    Title(String),
    Author(String),
    Date,
    Invalid,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FromParticipant(pub String);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ToParticipant(pub String);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct InteractionMessage(pub String);

// == Header ==============================================
#[derive(Debug)]
pub struct Header {}

// == Participant =========================================
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Participant {
    pub name: String,
    pub index: usize,
    pub active_from: usize,
    pub active_to: usize,
    // pub x: usize,
    // pub y: usize,
    // pub w: usize,
    // pub h: usize,
    pub rect: Rect,
}

// == Interaction Type ====================================
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum InteractionType {
    L2R,
    R2L,
    SelfRef,
}

// == Interaction =========================================
#[derive(Debug)]
pub struct Interaction {
    pub index: u32,
    pub from_participant: Participant,
    pub to_participant: Participant,
    pub interaction_type: InteractionType,
    pub message: Option<Message>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub input_source: Source,
    pub output_path: String,
}

#[derive(Debug, Clone)]
pub enum Source {
    StdIn,
    File(String),
    Example,
}
