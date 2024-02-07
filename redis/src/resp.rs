pub const TERMINATOR: &str = "\r\n";

#[derive(PartialEq, Debug)]
pub enum ParseError {
    InvalidPrefix,
    SyntaxError(SyntaxError),
}

#[derive(PartialEq, Debug)]
pub struct SyntaxError {
    pub message: String,
}

#[derive(PartialEq, Debug)]
pub enum Data {
    String(String),
    Error(Error),
    Integer(isize),
    BulkString(String),
    Array(Vec<ParseData>),
    Null,
    Boolean(bool),
}

#[derive(PartialEq, Debug)]
pub struct Error {
    pub message: String,
    pub kind: String,
}

#[derive(PartialEq, Debug)]
pub struct BulkString {
    pub length: usize,
    pub data: String,
}

pub type ParseData = (Data, usize);

pub type Result = std::result::Result<ParseData, ParseError>;
