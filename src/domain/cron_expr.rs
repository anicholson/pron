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
    let mut bits = 0u64;
    for element in s.split(',') {
        bits |= parse_element(element, min, max, name)?;
    }
    Ok(bits)
}

fn parse_element(s: &str, min: u32, max: u32, name: &str) -> Result<u64, ParseError> {
    if let Some(step_str) = s.strip_prefix("*/") {
        let step: u32 = step_str.parse().map_err(|_| {
            ParseError(format!("{} field: invalid step '{}'", name, s))
        })?;
        if step == 0 {
            return Err(ParseError(format!("{} field: invalid step '{}'", name, s)));
        }
        let mut bits = 0u64;
        let mut v = min;
        while v <= max {
            bits |= 1 << v;
            v += step;
        }
        return Ok(bits);
    }
    if s == "*" {
        let mut bits = 0u64;
        for v in min..=max {
            bits |= 1 << v;
        }
        return Ok(bits);
    }
    if let Some((lo_s, hi_s)) = s.split_once('-') {
        let lo: u32 = lo_s.parse().map_err(|_| {
            ParseError(format!("{} field: invalid value '{}'", name, s))
        })?;
        let hi: u32 = hi_s.parse().map_err(|_| {
            ParseError(format!("{} field: invalid value '{}'", name, s))
        })?;
        if lo < min || hi > max || lo > hi {
            return Err(ParseError(format!(
                "{} field: invalid value '{}'",
                name, s
            )));
        }
        let mut bits = 0u64;
        for v in lo..=hi {
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

        mod if_a_step_expression_has_an_invalid_step {
            #[test]
            fn then_a_parse_error_is_returned_naming_the_field() {
                let result = crate::domain::cron_expr::parse("*/0", "*", "*", "*", "*");
                assert!(result.is_err());
                let error = result.unwrap_err().to_string();
                assert!(
                    error.contains("minute"),
                    "error should name the field: {error}"
                );
                assert!(
                    error.contains("step"),
                    "error should name the step: {error}"
                );
            }
        }

        mod when_a_field_combines_ranges_and_lists {
            #[test]
            fn then_the_union_of_all_elements_is_set() {
                let expr = crate::domain::cron_expr::parse("1-5,10", "*", "*", "*", "*").unwrap();
                for m in [1, 2, 3, 4, 5, 10] {
                    assert!(
                        crate::domain::cron_expr::matches(&expr, m, 0, 1, 1, 0),
                        "minute {m} should match"
                    );
                }
                for m in [0, 6, 9, 11, 30, 59] {
                    assert!(
                        !crate::domain::cron_expr::matches(&expr, m, 0, 1, 1, 0),
                        "minute {m} should NOT match"
                    );
                }
            }
        }

        mod when_a_field_is_a_list_expression {
            #[test]
            fn then_every_listed_element_is_set() {
                let expr = crate::domain::cron_expr::parse("5,15,30", "*", "*", "*", "*").unwrap();
                assert!(crate::domain::cron_expr::matches(&expr, 5, 0, 1, 1, 0));
                assert!(crate::domain::cron_expr::matches(&expr, 15, 0, 1, 1, 0));
                assert!(crate::domain::cron_expr::matches(&expr, 30, 0, 1, 1, 0));
                assert!(!crate::domain::cron_expr::matches(&expr, 6, 0, 1, 1, 0));
                assert!(!crate::domain::cron_expr::matches(&expr, 31, 0, 1, 1, 0));
            }
        }

        mod when_a_field_is_a_range_expression {
            #[test]
            fn then_every_value_in_the_inclusive_range_is_set() {
                let expr = crate::domain::cron_expr::parse("10-12", "*", "*", "*", "*").unwrap();
                assert!(crate::domain::cron_expr::matches(&expr, 10, 0, 1, 1, 0));
                assert!(crate::domain::cron_expr::matches(&expr, 11, 0, 1, 1, 0));
                assert!(crate::domain::cron_expr::matches(&expr, 12, 0, 1, 1, 0));
                assert!(!crate::domain::cron_expr::matches(&expr, 9, 0, 1, 1, 0));
                assert!(!crate::domain::cron_expr::matches(&expr, 13, 0, 1, 1, 0));
            }
        }

        mod when_a_field_is_a_step_expression {
            #[test]
            fn then_only_every_nth_value_in_the_valid_range_is_set() {
                let expr = crate::domain::cron_expr::parse("*/15", "*", "*", "*", "*").unwrap();
                assert!(crate::domain::cron_expr::matches(&expr, 0, 0, 1, 1, 0));
                assert!(crate::domain::cron_expr::matches(&expr, 15, 0, 1, 1, 0));
                assert!(crate::domain::cron_expr::matches(&expr, 30, 0, 1, 1, 0));
                assert!(crate::domain::cron_expr::matches(&expr, 45, 0, 1, 1, 0));
                assert!(!crate::domain::cron_expr::matches(&expr, 10, 0, 1, 1, 0));
                assert!(!crate::domain::cron_expr::matches(&expr, 59, 0, 1, 1, 0));
            }
        }

        mod if_a_field_value_is_out_of_range {
            #[test]
            fn then_a_parse_error_is_returned_naming_the_field_and_value() {
                let result = crate::domain::cron_expr::parse("60", "*", "*", "*", "*");
                assert!(result.is_err());
                let error = result.unwrap_err().to_string();
                assert!(
                    error.contains("minute"),
                    "error should name the field: {error}"
                );
                assert!(
                    error.contains("60"),
                    "error should name the out-of-range value: {error}"
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