pub const TERMINATOR: &str = "\r\n";

pub mod error {
    #[derive(PartialEq, Debug)]
    pub enum RespError {
        InvalidPrefix,
        EmptyBuffer,
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
    pub data: Vec<Data>
}
