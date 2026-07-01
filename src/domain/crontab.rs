use crate::domain::cron_expr;

#[derive(Debug, Default)]
pub struct Entry {
    pub expr: cron_expr::CronExpr,
    pub command: String,
}

#[derive(Debug)]
pub struct ParseError(String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn parse(text: &str) -> Result<Vec<Entry>, ParseError> {
    let mut entries = vec![];
    for (i, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 6 {
            return Err(ParseError(format!(
                "line {}: expected 5 fields and a command",
                i + 1
            )));
        }
        let expr = cron_expr::parse(parts[0], parts[1], parts[2], parts[3], parts[4])
            .map_err(|e| ParseError(format!("line {}: {}", i + 1, e)))?;
        let command = parts[5..].join(" ");
        entries.push(Entry { expr, command });
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    mod parse {
        mod when_the_crontab_is_empty {
            #[test]
            fn then_no_entries_are_produced() {
                let result = crate::domain::crontab::parse("");
                assert!(result.is_ok());
                assert!(result.unwrap().is_empty());
            }
        }

        mod when_a_line_starts_with_hash {
            #[test]
            fn then_the_line_is_ignored() {
                let result = crate::domain::crontab::parse("# this is a comment\n");
                assert!(result.is_ok());
                assert!(result.unwrap().is_empty());
            }
        }

        mod when_a_line_is_blank {
            #[test]
            fn then_the_line_is_ignored() {
                let result = crate::domain::crontab::parse("   \n\n");
                assert!(result.is_ok());
                assert!(result.unwrap().is_empty());
            }
        }

        mod when_a_line_has_five_valid_fields_and_a_command {
            #[test]
            fn then_an_entry_is_produced_with_the_parsed_expression_and_whitespace_collapsed_command() {
                let result = crate::domain::crontab::parse("* * * * * echo    hi\n");
                assert!(result.is_ok(), "expected Ok, got Err: {:?}", result);
                let entries = result.unwrap();
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].command, "echo hi");
            }
        }

        mod if_a_line_has_an_invalid_field_value {
            #[test]
            fn then_a_parse_error_is_returned_naming_the_line_and_field() {
                let result = crate::domain::crontab::parse("not * * * * echo hi\n");
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

        mod if_a_line_has_fewer_than_five_fields {
            #[test]
            fn then_a_parse_error_is_returned_naming_the_line() {
                let result = crate::domain::crontab::parse("* * * echo hi\n");
                assert!(result.is_err());
                let error = result.unwrap_err().to_string();
                assert!(
                    error.contains("line 1"),
                    "error should name the line: {error}"
                );
                assert!(
                    error.contains("5 fields"),
                    "error should name the expected field count: {error}"
                );
            }
        }

        mod if_a_line_has_exactly_five_fields_and_a_command {
            #[test]
            fn then_the_boundary_between_valid_and_invalid_is_pinned() {
                let ok = crate::domain::crontab::parse("0 0 1 1 0 echo hi\n");
                assert!(ok.is_ok(), "6 parts should succeed: {:?}", ok);
                assert_eq!(ok.unwrap().len(), 1);

                let err = crate::domain::crontab::parse("0 0 1 1 echo hi\n");
                assert!(err.is_err(), "5 parts should fail");
                let error = err.unwrap_err().to_string();
                assert!(
                    error.contains("5 fields"),
                    "error should name the field count: {error}"
                );
            }
        }
    }
}