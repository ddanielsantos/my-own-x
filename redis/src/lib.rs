mod resp;

pub fn parse(buffer: &str) -> Result<resp::Data, resp::error::RespError> {
    if buffer.is_empty() {
        return Err(resp::error::RespError::EmptyBuffer)
    }

    let first_byte = get_first_byte(&buffer);
    let input = remove_terminator(skip_fist_byte(buffer));

    match first_byte {
        "+" => {
            Ok(resp::Data::String(input.join("")))
        }
        "-" => {
            let st = input.join("");
            let mut split = st.splitn(2, " ");

            Ok(resp::Data::Error(resp::Error {
                kind: split.next().unwrap_or_default().to_string(),
                message: split.next().unwrap_or_default().to_string()
            }))
        }
        ":" => {
            let parse_result: usize = input.get(0).unwrap().parse().unwrap();
            Ok(resp::Data::Integer(parse_result))
        }
        "$" => {
            let blk = resp::BulkString {
                length: input.get(0).unwrap().to_string().parse().unwrap(),
                data: input.get(1).unwrap().to_string(),
            };

            Ok(resp::Data::BulkString(blk))
        }
        // "*" => {
        //     Data::parse(input, DataKind::Array)
        // }
        // "_" => {
        //     Data::parse(input, DataKind::Null)
        // }
        // "#" => {
        //     Data::parse(input, DataKind::Boolean)
        // }
        // "," => {
        //     Data::parse(input, DataKind::Double)
        // }
        // "(" => {
        //     Data::parse(input, DataKind::BigNumber)
        // }
        // "!" => {
        //     Data::parse(input, DataKind::BulkError)
        // }
        // "=" => {
        //     Data::parse(input, DataKind::VerbatimString)
        // }
        // "%" => {
        //     Data::parse(input, DataKind::Map)
        // }
        // "~" => {
        //     Data::parse(input, DataKind::Set)
        // }
        // ">" => {
        //     Data::parse(input, DataKind::Push)
        // }
        _ => {
            Err(resp::error::RespError::InvalidPrefix)
        }
    }
}

fn get_first_byte(buffer: &str) -> &str {
    &buffer[..1]
}

fn skip_fist_byte(buffer: &str) -> &str {
    &buffer[1..]
}

fn remove_terminator(buffer: &str) -> Vec<&str> {
    buffer
        .split(resp::TERMINATOR)
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::resp::{BulkString, Data, Error};

    #[test]
    pub fn parse_string() {
        let expected = Data::String("hello world".to_string());
        let result = crate::parse("+hello world\r\n");

        assert_eq!(result, Ok(expected));
    }

    #[test]
    pub fn parse_error() {
        let expected = Data::Error(Error {
            kind: "Error".to_string(),
            message: "message".to_string()
        });
        let result = crate::parse("-Error message\r\n");

        assert_eq!(result, Ok(expected));
    }

    #[test]
    pub fn parse_integer() {
        let expected = Data::Integer(10);
        let result = crate::parse(":10\r\n");

        assert_eq!(result, Ok(expected));
    }

    #[test]
    pub fn parse_bulk_string() {
        let expected = Data::BulkString(BulkString {
            length: 5,
            data: "hello".to_string(),
        });
        let result = crate::parse("$5\r\nhello\r\n");

        assert_eq!(result, Ok(expected));
    }
}