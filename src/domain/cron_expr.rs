#[derive(Debug, Default)]
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
    match s.parse::<u32>() {
        Ok(v) if (min..=max).contains(&v) => Ok(1 << v),
        _ => Err(ParseError(format!("{} field: invalid value '{}'", name, s))),
    }
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

pub fn matches(expr: &CronExpr, min: u32, hour: u32, dom: u32, mon: u32, dow: u32) -> bool {
    (expr.minute & (1 << min) != 0)
        && (expr.hour & (1 << hour) != 0)
        && (expr.dom & (1 << dom) != 0)
        && (expr.month & (1 << mon) != 0)
        && (expr.dow & (1 << dow) != 0)
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
        mod when_called_with_a_tuple_that_matches_a_fully_numeric_expression {
            #[test]
            fn then_true_is_returned() {
                let expr = crate::domain::cron_expr::parse("5", "3", "15", "7", "2").unwrap();
                assert!(crate::domain::cron_expr::matches(&expr, 5, 3, 15, 7, 2));
            }

            #[test]
            fn and_false_is_returned_for_a_one_off_minute() {
                let expr = crate::domain::cron_expr::parse("5", "3", "15", "7", "2").unwrap();
                assert!(!crate::domain::cron_expr::matches(&expr, 6, 3, 15, 7, 2));
            }

            #[test]
            fn and_false_is_returned_for_a_one_off_hour() {
                let expr = crate::domain::cron_expr::parse("5", "3", "15", "7", "2").unwrap();
                assert!(!crate::domain::cron_expr::matches(&expr, 5, 4, 15, 7, 2));
            }

            #[test]
            fn and_false_is_returned_for_a_one_off_day_of_month() {
                let expr = crate::domain::cron_expr::parse("5", "3", "15", "7", "2").unwrap();
                assert!(!crate::domain::cron_expr::matches(&expr, 5, 3, 16, 7, 2));
            }

            #[test]
            fn and_false_is_returned_for_a_one_off_month() {
                let expr = crate::domain::cron_expr::parse("5", "3", "15", "7", "2").unwrap();
                assert!(!crate::domain::cron_expr::matches(&expr, 5, 3, 15, 8, 2));
            }

            #[test]
            fn and_false_is_returned_for_a_one_off_day_of_week() {
                let expr = crate::domain::cron_expr::parse("5", "3", "15", "7", "2").unwrap();
                assert!(!crate::domain::cron_expr::matches(&expr, 5, 3, 15, 7, 3));
            }
        }

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