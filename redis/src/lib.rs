mod resp;

pub fn parse(buffer: &str) -> resp::Result {
    let first_byte = get_first_byte(buffer);

    parse_internal_1(first_byte, &buffer)
}

fn parse_string(buffer: &str) -> resp::Result {
    let b = skip_fist_byte(buffer);
    let b = b.replace(resp::TERMINATOR, "");
    Ok(resp::Data::String(b))
}

fn parse_error(buffer: &str) -> resp::Result {
    let b = skip_fist_byte(buffer);
    let b = b.replace(resp::TERMINATOR, "");
    let mut b = b.splitn(2, ' ');
    Ok(resp::Data::Error(resp::Error {
        kind: b.next().unwrap().to_string(),
        message: b.next().unwrap().to_string(),
    }))
}

fn parse_integer(buffer: &str) -> resp::Result {
    dbg!(&buffer);
    let b = skip_fist_byte(buffer);

    if !b.ends_with(resp::TERMINATOR) {
        return Err(vec![resp::error::RespError::SyntaxError(
            resp::error::SyntaxError {
                message: "Invalid buffer, it should be terminated with \r\n".to_string(),
            },
        )]);
    }

    let b = b.replace(resp::TERMINATOR, "");
    dbg!(&b);
    Ok(resp::Data::Integer(b.parse().unwrap()))
}

fn parse_bulk_string(buffer: &str) -> resp::Result {
    let b = skip_fist_byte(buffer);
    let mut b = b
        .splitn(2, resp::TERMINATOR)
        .map(|s| s.replace(resp::TERMINATOR, ""));
    Ok(resp::Data::BulkString(resp::BulkString {
        length: b.next().unwrap().parse().unwrap(),
        data: b.next().unwrap().to_string(),
    }))
}

fn parse_array(buffer: &str) -> resp::Result {
    let b = skip_fist_byte(buffer);
    dbg!(&b);
    let mut b = b.splitn(2, resp::TERMINATOR).into_iter();

    let length: usize = b.next().unwrap().parse().unwrap();

    if length == 0 {
        return Ok(resp::Data::Array(resp::Array {
            length,
            data: vec![],
        }));
    }

    let b = b.next().unwrap();
    dbg!(&b); // TODO tentar pegar o proximo chunk

    let data: Vec<resp::Data> = b
        .split_inclusive(resp::TERMINATOR)
        .filter_map(|b| parse(b).ok())
        .collect();

    Ok(resp::Data::Array(resp::Array { length, data }))
}

fn parse_internal_1<'a>(first_byte: &str, buffer: &str) -> resp::Result {
    match first_byte {
        "+" => parse_string(buffer),
        "-" => parse_error(buffer),
        ":" => parse_integer(buffer),
        "$" => parse_bulk_string(buffer),
        "*" => parse_array(buffer),
        e => {
            dbg!(&e);
            Err(vec![resp::error::RespError::InvalidPrefix])
        }
    }
}

fn get_first_byte(buffer: &str) -> &str {
    &buffer[..1]
}

fn skip_fist_byte(buffer: &str) -> &str {
    &buffer[1..]
}

fn split_at_terminator(buffer: &str, inclusive: bool) -> Vec<&str> {
    if inclusive {
        return buffer.split_inclusive(resp::TERMINATOR).collect();
    }
    buffer.split(resp::TERMINATOR).collect()
}

#[cfg(test)]
mod tests {
    use crate::{
        resp,
        resp::{BulkString, Data, Error},
    };

    pub fn assert_len(expected: usize, result: resp::Result) {
        match result {
            Ok(Data::Array(arr)) => {
                assert_eq!(expected, arr.length);
                assert_eq!(arr.data.len(), arr.length)
            }
            // TODO add support to maps and sets
            _ => {
                panic!("Result was not an array")
            }
        }
    }

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
            message: "message".to_string(),
        });
        let result = crate::parse("-Error message\r\n");

        assert_eq!(result, Ok(expected));
    }

    mod integer {
        use crate::resp;
        use crate::resp::error::{RespError, SyntaxError};
        use crate::resp::Data;

        #[test]
        pub fn parse_integer() {
            let expected = Data::Integer(10);
            let result = crate::parse(":10\r\n");

            assert_eq!(result, Ok(expected));
        }

        #[test]
        pub fn parse_integer_error() {
            let expected: resp::Result = Err(vec![RespError::SyntaxError(SyntaxError {
                message: "Invalid buffer, it should be terminated with \r\n".to_string(),
            })]);
            let result = crate::parse(":10");

            assert_eq!(result, expected);
        }
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

    mod arrays {
        use crate::{
            resp::{Array, Data},
            tests::assert_len,
        };

        #[test]
        pub fn parse_array() {
            let expected = Data::Array(Array {
                length: 3,
                data: vec![Data::Integer(1), Data::Integer(2), Data::Integer(3)],
            });

            let result = crate::parse("*3\r\n:1\r\n:2\r\n:3\r\n");

            assert_eq!(result, Ok(expected));
            assert_len(3, result);
        }

        #[test]
        pub fn parse_empty_array() {
            let expected = Data::Array(Array {
                length: 0,
                data: vec![],
            });

            let result = crate::parse("*0\r\n");

            assert_eq!(result, Ok(expected));
            assert_len(0, result);
        }

        #[test]
        pub fn parse_array_with_different_types() {
            let expected = Data::Array(Array {
                length: 3,
                data: vec![
                    Data::Integer(1),
                    Data::String("awesome test".to_string()),
                    Data::Integer(3),
                ],
            });

            let result = crate::parse("*3\r\n:1\r\n+awesome test\r\n:3\r\n");

            assert_eq!(result, Ok(expected));
            assert_len(3, result);
        }

        #[test]
        pub fn parse_nested_array() {
            let expected = Data::Array(Array {
                length: 3,
                data: vec![
                    Data::Integer(1),
                    Data::Array(Array {
                        length: 1,
                        data: vec![Data::Integer(2)],
                    }),
                    Data::Array(Array {
                        length: 1,
                        data: vec![Data::Integer(3)],
                    }),
                ],
            });

            let result = crate::parse("*3\r\n:1\r\n*1\r\n:2\r\n*1\r\n:3\r\n");

            assert_eq!(result, Ok(expected));
            assert_len(3, result);
        }
    }
}
