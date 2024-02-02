use crate::resp::{Data, Error, ParseData, ParseError, SyntaxError};

mod resp;

fn parse(buffer: &str) -> resp::Result {
    match &buffer[..1] {
        "+" => parse_string(buffer),
        ":" => parse_integer(buffer),
        "-" => parse_error(buffer),
        "*" => parse_array(buffer),
        _ => {
            Err(ParseError::InvalidPrefix)
        }
    }
}

fn parse_string(buffer: &str) -> resp::Result {
    match buffer.find(resp::TERMINATOR) {
        Some(term_index) => {
            let val = (&buffer[1..term_index]).to_string();
            Ok((Data::String(val), term_index + 2))
        }
        None => {
            Err(ParseError::InvalidPrefix)
        }
    }
}

fn parse_integer(buffer: &str) -> resp::Result {
    match buffer.find(resp::TERMINATOR) {
        Some(term_index) => {
            let value = (&buffer[1..term_index]).parse().unwrap();

            Ok((Data::Integer(value), term_index + 2))
        }
        None => {
            Err(ParseError::InvalidPrefix)
        }
    }
}

fn parse_error(buffer: &str) -> resp::Result {
    match buffer.find(resp::TERMINATOR) {
        Some(term_index) => {
            match buffer.find(|c: char| c.is_whitespace()) {
                Some(whitespace_index) => {
                    let val = Error {
                        kind: (&buffer[1..whitespace_index]).to_string(),
                        message: (&buffer[whitespace_index + 1..term_index]).to_string(),
                    };

                    Ok((Data::Error(val), term_index + 2))
                }
                None => {
                    Err(ParseError::SyntaxError(SyntaxError { message: "No whitespace found for Error variant".to_string() }))
                }
            }
        }
        None => {
            Err(ParseError::InvalidPrefix)
        }
    }
}

fn parse_array(buffer: &str) -> resp::Result {
    match buffer.find(resp::TERMINATOR) {
        Some(term_index) => {
            let length = buffer[1..term_index].parse().unwrap();
            let mut start_index = term_index + 2;

            let mut data: Vec<ParseData> = vec![];
            for _ in 0..length {
                match parse(&buffer[start_index..]) {
                    Ok(p) => {
                        start_index += &p.1;
                        data.push(p);
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }

            Ok((Data::Array(data), term_index + 2))
        }
        None => {
            Err(ParseError::InvalidPrefix)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{resp, resp::Data};

    #[test]
    pub fn parse_string() {
        let expected = Data::String("hello world".to_string());
        let result = crate::parse("+hello world\r\n");

        assert_eq!(result, Ok((expected, 14)));
    }

    #[test]
    pub fn parse_array() {
        let expected = Data::Array(vec![(Data::String("zap".to_string()), 6)]);
        let result = crate::parse("*1\r\n+zap\r\n");

        assert_eq!(result, Ok((expected, 4)));
    }

    #[test]
    pub fn parse_array_with_different_types() {
        let expected = Data::Array(vec![
            (Data::Integer(1), 4),
            (Data::String("awesome test".to_string()), 15),
            (Data::Integer(3), 4),
        ]);

        let result = crate::parse("*3\r\n:1\r\n+awesome test\r\n:3\r\n");

        assert_eq!(result, Ok((expected, 4)));
    }

    #[test]
    pub fn parse_nested_array() {
        let expected = Data::Array(vec![
            (Data::String("looks good".to_string()), 13),
            (Data::Array(vec![(Data::Integer(1), 4), (Data::Integer(2), 4), (Data::Integer(3), 4)]), 4),
            (Data::Integer(1), 4),
        ]);

        let result = crate::parse("*3\r\n+looks good\r\n*3\r\n:1\r\n:2\r\n:3\r\n*2\r\n+Foo\r\n-Bar\r\n");

        assert_eq!(result, Ok((expected, 4)));
    }
    #[test]
    pub fn parse_error() {
        let expected = Data::Error(resp::Error{ message: "message".to_string(), kind: "Error".to_string() });
        let result = crate::parse("-Error message\r\n");

        assert_eq!(result, Ok((expected, 16)));
    }
}
