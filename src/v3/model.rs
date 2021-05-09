// == Message =============================================
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub struct Message(pub String);

// == Line ================================================
#[derive(Debug)]
pub struct Line {
    pub line_number: u32,
    pub line_contents: LineContents,
    pub line_data: String,
}

// == Line Contents =======================================
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum LineContents {
    Invalid,
    Nothing,
    Comment,
    MetaData,
    Interaction,
    InteractionWithMessage,
}

// == Header ==============================================
#[derive(Debug)]
pub struct Header {}

// == Participant =========================================
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Participant {
    pub name: String,
    pub index: usize,
    pub active_from: i32,
    pub active_to: i32,
}

// == Interaction Type ====================================
#[derive(Debug)]
pub enum InteractionType {
    L2R,
    // R2L,
    // SELF,
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
