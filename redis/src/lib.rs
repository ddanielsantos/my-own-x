const TERMINATOR: &str = "\r\n";

#[derive(PartialEq,  Debug)]
pub enum Data {
    String(String),
    Error(Error),
    Integer(usize),
    BulkString(BulkString),
    Array,
    // Null,
    Boolean,
    Double,
    BigNumber,
    BulkError,
    VerbatimString,
    Map,
    Set,
    Push
}

#[derive(PartialEq,  Debug)]
pub struct Error {
    message: String,
    kind: String,
}

#[derive(PartialEq,  Debug)]
pub struct BulkString {
    length: usize,
    data: String,
}

pub fn deser(input: &str) -> Data {
    let first_byte = get_first_byte(&input);
    let input = remove_terminator(skip_fist_byte(input));

    match first_byte {
        "+" => {
            Data::String(input.join(""))
        }
        "-" => {
            let st = input.join("");
            let mut split = st.splitn(2, " ");

            Data::Error(Error {
                kind: split.next().unwrap_or_default().to_string(),
                message: split.next().unwrap_or_default().to_string()
            })
        }
        ":" => {
            let parse_result: usize = input.get(0).unwrap().parse().unwrap();
            Data::Integer(parse_result)
        }
        "$" => {
            let blk = BulkString {
                length: input.get(0).unwrap().to_string().parse().unwrap(),
                data: input.get(1).unwrap().to_string(),
            };

            Data::BulkString(blk)
        }
        // "*" => {
        //     Data::deser(input, DataKind::Array)
        // }
        // "_" => {
        //     Data::deser(input, DataKind::Null)
        // }
        // "#" => {
        //     Data::deser(input, DataKind::Boolean)
        // }
        // "," => {
        //     Data::deser(input, DataKind::Double)
        // }
        // "(" => {
        //     Data::deser(input, DataKind::BigNumber)
        // }
        // "!" => {
        //     Data::deser(input, DataKind::BulkError)
        // }
        // "=" => {
        //     Data::deser(input, DataKind::VerbatimString)
        // }
        // "%" => {
        //     Data::deser(input, DataKind::Map)
        // }
        // "~" => {
        //     Data::deser(input, DataKind::Set)
        // }
        // ">" => {
        //     Data::deser(input, DataKind::Push)
        // }
        _ => {
            Data::String("".to_string())
        }
    }
}

fn get_first_byte(input: &str) -> &str {
    &input[..1]
}

fn skip_fist_byte(input: &str) -> &str {
    &input[1..]
}

fn remove_terminator(input: &str) -> Vec<&str> {
    input
        .split(TERMINATOR)
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{BulkString, Data, Error};

    #[test]
    pub fn deserialize_string() {
        let expected = Data::String("hello world".to_string());
        let result = crate::deser("+hello world\r\n");

        assert_eq!(result, expected);
    }

    #[test]
    pub fn deserialize_error() {
        let expected = Data::Error(Error {
            kind: "Error".to_string(),
            message: "message".to_string()
        });
        let result = crate::deser("-Error message\r\n");

        assert_eq!(result, expected);
    }

    #[test]
    pub fn deserialize_integer() {
        let expected = Data::Integer(10);
        let result = crate::deser(":10\r\n");

        assert_eq!(result, expected);
    }

    #[test]
    pub fn deserialize_bulk_string() {
        let expected = Data::BulkString(BulkString {
            length: 5,
            data: "hello".to_string(),
        });
        let result = crate::deser("$5\r\nhello\r\n");

        assert_eq!(result, expected);
    }
}