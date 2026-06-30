pub struct Entry;

#[derive(Debug)]
pub struct ParseError(String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn parse(_text: &str) -> Result<Vec<Entry>, ParseError> {
    Ok(vec![])
}

#[cfg(test)]
mod tests {
    mod parse {
        mod if_a_line_has_an_invalid_field_value {
            #[test]
            fn then_a_parse_error_is_returned_naming_the_line_and_field() {
                let result = crate::domain::crontab::parse("not a valid cron line\n");
                assert!(result.is_err());
                let error = result.unwrap_err().to_string();
                assert!(
                    error.contains("line 1"),
                    "error should name the line: {error}"
                );
                assert!(
                    error.contains("field"),
                    "error should name the field: {error}"
                );
            }
        }
    }
}
