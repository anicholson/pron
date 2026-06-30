#[derive(Debug)]
pub struct CronExpr;

#[derive(Debug)]
pub struct ParseError(String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn parse_field(s: &str, name: &str) -> Result<(), ParseError> {
    if s == "*" {
        return Ok(());
    }
    Err(ParseError(format!("{} field: invalid value '{}'", name, s)))
}

pub fn parse(
    minute: &str,
    hour: &str,
    dom: &str,
    month: &str,
    dow: &str,
) -> Result<CronExpr, ParseError> {
    parse_field(minute, "minute")?;
    parse_field(hour, "hour")?;
    parse_field(dom, "day-of-month")?;
    parse_field(month, "month")?;
    parse_field(dow, "day-of-week")?;
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

    mod matches {
        mod when_called_with_a_minute_tuple_that_matches {
            #[test]
            fn then_true_is_returned() {
                let expr = crate::domain::cron_expr::parse("*", "*", "*", "*", "*").unwrap();
                assert!(crate::domain::cron_expr::matches(&expr, 0, 0, 1, 1, 0));
            }
        }

        mod when_called_with_a_minute_tuple_that_does_not_match {
            #[test]
            fn then_false_is_returned() {
                let expr = crate::domain::cron_expr::parse("0", "*", "*", "*", "*").unwrap();
                assert!(!crate::domain::cron_expr::matches(&expr, 30, 0, 1, 1, 0));
            }
        }
    }
}
