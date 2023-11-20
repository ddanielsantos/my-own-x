pub const TERMINATOR: &str = "\r\n";
pub const AGGREGATE_PREFIXES: &[&str] = &["*"];

pub mod error {
    #[derive(PartialEq, Debug)]
    pub enum RespError {
        InvalidPrefix,
        EmptyBuffer,
        SyntaxError(SyntaxError),
    }

    #[derive(PartialEq, Debug)]
    pub struct SyntaxError {
        pub message: String,
    }
}

#[derive(PartialEq, Debug)]
pub enum Data {
    String(String),
    Error(Error),
    Integer(usize),
    BulkString(BulkString),
    Array(Array),
    Boolean,
    Double,
    BigNumber,
    BulkError,
    VerbatimString,
    Map,
    Set,
    Push,
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

#[derive(PartialEq, Debug)]
pub struct Array {
    pub length: usize,
    pub data: Vec<Data>,
}

pub type Result = std::result::Result<Data, Vec<error::RespError>>;