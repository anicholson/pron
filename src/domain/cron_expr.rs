#[derive(Debug)]
pub struct CronExpr {
    minute: u64,
    hour: u64,
    dom: u64,
    month: u64,
    dow: u64,
}

#[derive(Debug)]
pub struct ParseError(String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn parse_field(s: &str, min: u32, max: u32, name: &str) -> Result<u64, ParseError> {
    if s == "*" {
        let mut bits = 0u64;
        for v in min..=max {
            bits |= 1 << v;
        }
        return Ok(bits);
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
    Ok(CronExpr {
        minute: parse_field(minute, 0, 59, "minute")?,
        hour: parse_field(hour, 0, 23, "hour")?,
        dom: parse_field(dom, 1, 31, "day-of-month")?,
        month: parse_field(month, 1, 12, "month")?,
        dow: parse_field(dow, 0, 6, "day-of-week")?,
    })
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
