#[derive(Debug)]
pub struct CronExpr;

#[derive(Debug)]
pub struct ParseError(String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn parse(
    _minute: &str,
    _hour: &str,
    _dom: &str,
    _month: &str,
    _dow: &str,
) -> Result<CronExpr, ParseError> {
    Ok(CronExpr)
}

#[cfg(test)]
mod tests {
    mod parse {
        mod if_a_field_value_is_invalid {
            #[test]
            fn then_a_parse_error_is_returned_naming_the_field_and_value() {
                let result = crate::domain::cron_expr::parse("not", "*", "*", "*", "*");
                assert!(result.is_err());
                let error = result.unwrap_err().to_string();
                assert!(
                    error.contains("minute"),
                    "error should name the field: {error}"
                );
                assert!(
                    error.contains("not"),
                    "error should name the invalid value: {error}"
                );
            }
        }
    }
}
