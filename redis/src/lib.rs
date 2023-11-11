const TERMINATOR: &str = "\r\n";

#[derive(PartialEq,  Debug)]
pub enum DataKind {
    String,
    Error,
    Integer,
    BulkString,
    Array,
    Null,
    Boolean,
    Double,
    BigNumber,
    BulkError,
    VerbatimString,
    Map,
    Set,
    Push
}

pub struct Data {
    value: String,
    kind: DataKind
}

impl Data {
    pub fn deser(input: String, kind: DataKind) -> Data {
        Data {
            value: input,
            kind
        }
    }
}

pub fn deser(input: &str) -> Data {
    let first_byte = get_first_byte(&input);
    let input = remove_terminator(skip_fist_byte(input));

    match first_byte {
        "+" => {
            Data::deser(input, DataKind::String)
        }
        "-" => {
            Data::deser(input, DataKind::Error)
        }
        ":" => {
            Data::deser(input, DataKind::Integer)
        }
        "$" => {
            Data::deser(input, DataKind::BulkString)
        }
        "*" => {
            Data::deser(input, DataKind::Array)
        }
        "_" => {
            Data::deser(input, DataKind::Null)
        }
        "#" => {
            Data::deser(input, DataKind::Boolean)
        }
        "," => {
            Data::deser(input, DataKind::Double)
        }
        "(" => {
            Data::deser(input, DataKind::BigNumber)
        }
        "!" => {
            Data::deser(input, DataKind::BulkError)
        }
        "=" => {
            Data::deser(input, DataKind::VerbatimString)
        }
        "%" => {
            Data::deser(input, DataKind::Map)
        }
        "~" => {
            Data::deser(input, DataKind::Set)
        }
        ">" => {
            Data::deser(input, DataKind::Push)
        }
        _ => {
            Data::deser("".to_string(), DataKind::Null)
        }
    }
}

fn get_first_byte(input: &str) -> &str {
    &input[..1]
}

fn skip_fist_byte(input: &str) -> &str {
    &input[1..]
}

fn remove_terminator(input: &str) -> String {
    input
        .replace(TERMINATOR, " ")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use crate::DataKind;

    #[test]
    pub fn should_deserialize_strings() {
        let expected = "hello world";
        let result = crate::deser("+hello world\r\n");

        assert_eq!(result.value, expected);
        assert_eq!(result.kind, DataKind::String);
    }

}