use crate::resp::{Data, Error, ParseData, ParseError, SyntaxError};

mod resp;

fn parse(buffer: &str) -> resp::Result {
    let first_byte = &buffer.chars().nth(0).ok_or(ParseError::InvalidPrefix)?;
    match first_byte {
        '+' => parse_string(buffer),
        ':' => parse_integer(buffer),
        '-' => parse_error(buffer),
        '*' => parse_array(buffer),
        '$' => parse_bulk_string(buffer),
        _ => Err(ParseError::InvalidPrefix)
    }
}

fn parse_string(buffer: &str) -> resp::Result {
    let term_index = get_terminator_index(buffer)?;
    let data = (&buffer[1..term_index]).to_string();
    Ok((Data::String(data), term_index + 2))
}

fn parse_integer(buffer: &str) -> resp::Result {
    let term_index = get_terminator_index(buffer)?;
    let data = (&buffer[1..term_index]).parse().unwrap();
    Ok((Data::Integer(data), term_index + 2))
}

fn parse_error(buffer: &str) -> resp::Result {
    let term_index = get_terminator_index(buffer)?;
    let whitespace_index = get_whitespace_index(buffer)?;
    let data = Error {
        kind: (&buffer[1..whitespace_index]).to_string(),
        message: (&buffer[whitespace_index + 1..term_index]).to_string(),
    };

    Ok((Data::Error(data), term_index + 2))
}

fn get_whitespace_index(buffer: &str) -> Result<usize, ParseError> {
    buffer.find(|c: char| c.is_whitespace())
        .ok_or(ParseError::SyntaxError(SyntaxError { message: "No whitespace found".to_string() }))
}

fn parse_array(buffer: &str) -> resp::Result {
    let term_index = get_terminator_index(buffer)?;
    let length = buffer[1..term_index].parse().unwrap();
    let mut bytes_consumed = term_index + 2;

    let mut data: Vec<ParseData> = vec![];
    for _ in 0..length {
        let parse_data = parse(&buffer[bytes_consumed..])?;
        bytes_consumed += &parse_data.1;
        data.push(parse_data);
    }

    Ok((Data::Array(data), bytes_consumed))
}

fn get_terminator_index(buffer: &str) -> Result<usize, ParseError> {
    buffer.find(resp::TERMINATOR).ok_or(ParseError::SyntaxError(SyntaxError { message: "Could not find terminator".to_string() }))
}

fn get_rterminator_index(buffer: &str) -> Result<usize, ParseError> {
    buffer.rfind(resp::TERMINATOR).ok_or(ParseError::SyntaxError(SyntaxError { message: "Could not find closing terminator".to_string() }))
}

fn parse_bulk_string(buffer: &str) -> resp::Result {
    let first_terminator_index = get_terminator_index(buffer)?;
    let last_terminator_index = get_rterminator_index(buffer)?;
    let data = &buffer[first_terminator_index + 2..last_terminator_index];
    Ok((Data::BulkString(data.to_string()), data.len()))
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
    pub fn parse_negative_integer() {
        let expected = Data::Integer(-2);
        let result = crate::parse(":-2\r\n");

        assert_eq!(result, Ok((expected, 5)));
    }

    #[test]
    pub fn parse_array() {
        let expected = Data::Array(vec![(Data::String("zap".to_string()), 6)]);
        let result = crate::parse("*1\r\n+zap\r\n");

        assert_eq!(result, Ok((expected, 10)));
    }

    #[test]
    pub fn parse_array_with_different_types() {
        let expected = Data::Array(vec![
            (Data::Integer(1), 4),
            (Data::String("awesome test".to_string()), 15),
            (Data::Integer(3), 4),
        ]);

        let result = crate::parse("*3\r\n:1\r\n+awesome test\r\n:3\r\n");

        assert_eq!(result, Ok((expected, 27)));
    }

    #[test]
    pub fn parse_nested_array() {
        let expected = Data::Array(vec![
            (Data::String("looks good".to_string()), 13),
            (Data::Array(vec![(Data::Integer(1), 4), (Data::Integer(2), 4), (Data::Integer(3), 4)]), 16),
            (Data::Array(vec![(Data::String("Foo".to_string()), 6), (Data::Error(resp::Error { kind: "Bar".to_string(), message: "baz".to_string() }), 10)]), 20),
        ]);

        let result = crate::parse("*3\r\n+looks good\r\n*3\r\n:1\r\n:2\r\n:3\r\n*2\r\n+Foo\r\n-Bar baz\r\n");

        assert_eq!(result, Ok((expected, 53)));
    }

    #[test]
    pub fn parse_empty_array() {
        let expected = Data::Array(vec![]);

        let result = crate::parse("*0\r\n");

        assert_eq!(result, Ok((expected, 4)));
    }

    #[test]
    pub fn parse_tricky_empty_array() {
        let expected = Data::Array(vec![]);

        let result = crate::parse("*0\r\n+hello\r\n");

        assert_eq!(result, Ok((expected, 4)));
    }

    #[test]
    pub fn parse_error() {
        let expected = Data::Error(resp::Error{ message: "message".to_string(), kind: "Error".to_string() });
        let result = crate::parse("-Error message\r\n");

        assert_eq!(result, Ok((expected, 16)));
    }

    #[test]
    pub fn parse_bulk_string() {
        let expected = Data::BulkString("hello".to_string());
        let result = crate::parse("$5\r\nhello\r\n");

        assert_eq!(result, Ok((expected, 5)));
    }

    #[test]
    pub fn parse_empty_bulk_string() {
        let expected = Data::BulkString("".to_string());
        let result = crate::parse("$0\r\n\r\n");

        assert_eq!(result, Ok((expected, 0)));
    }
}
