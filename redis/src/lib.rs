mod resp;

pub fn parse(buffer: &str) -> resp::Result {
    dbg!(&buffer);
    if buffer.is_empty() {
        return Err(vec![resp::error::RespError::EmptyBuffer]);
    }

    let (first_byte, rest) = split_at_first_byte(buffer);

    if let Some(val) = rest.last() {
        if !val.is_empty() && first_byte != "*" {
            return Err(vec![resp::error::RespError::SyntaxError(resp::error::SyntaxError {
                message: "Invalid buffer, it should be terminated with \r\n".to_string(),
            })]);
        }
    }

    parse_internal(first_byte, &rest)
}

fn parse_internal<'a>(
    first_byte: &str,
    rest: &Vec<&str>,
) -> resp::Result {
    match first_byte {
        "+" => Ok(resp::Data::String(rest.join(""))),
        "-" => {
            let st = rest.join("");
            let mut split = st.splitn(2, ' ');

            Ok(resp::Data::Error(resp::Error {
                kind: split.next().unwrap_or_default().to_string(),
                message: split.next().unwrap_or_default().to_string(),
            }))
        }
        ":" => {
            let parse_result: usize = rest.first().unwrap().parse().unwrap();
            Ok(resp::Data::Integer(parse_result))
        }
        "$" => {
            let blk = resp::BulkString {
                length: rest.first().unwrap().to_string().parse().unwrap(),
                data: rest.get(1).unwrap().to_string(),
            };

            Ok(resp::Data::BulkString(blk))
        }
        "*" => {
            let mut input = rest.iter();

            let length = input
                .next()
                .unwrap() // TODO: SyntaxError
                .replace(resp::TERMINATOR, "")
                .parse() // TODO: ParseError
                .unwrap();

            dbg!(&length);
            dbg!(&rest);

            let data = input
                .map(|i| {
                    let (first_byte, rest) = split_at_first_byte(i);

                    parse_internal(first_byte, &rest)
                });

            let errors: Vec<resp::error::RespError> = data.clone()
                .filter_map(|res| res.err())
                .flatten()
                .collect();

            if !errors.is_empty() {
                return Err(errors);
            }

            let data: Vec<resp::Data> = data.filter_map(|res| res.ok())
                .collect();

            Ok(resp::Data::Array(resp::Array { length, data }))
        }
        e => {
            dbg!(&e);
            Err(vec![resp::error::RespError::InvalidPrefix])
        }
    }
}

fn split_at_first_byte(buffer: &str) -> (&str, Vec<&str>) {
    let first_byte = get_first_byte(buffer);
    let inclusive = first_byte == "*";
    let input = split_at_terminator(skip_fist_byte(buffer), inclusive);
    (first_byte, input)
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
        resp::{BulkString, Data, Error}
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
                length: 2,
                data: vec![
                    Data::Integer(1),
                    Data::Array(Array {
                        length: 2,
                        data: vec![Data::Integer(2), Data::Integer(3)],
                    }),
                ],
            });

            let result = crate::parse("*2\r\n:1\r\n*2\r\n:2\r\n:3\r\n");

            assert_eq!(result, Ok(expected));
            assert_len(result);
        }
    }
}
