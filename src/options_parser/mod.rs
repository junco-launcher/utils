use std::fmt;

/// Represents the possible data types that can be parsed from an options line.
#[derive(Debug, Clone, PartialEq)]
pub enum OptionsDataType {
    /// An integer value.
    Integer(i64),
    /// A floating-point value.
    Float(f64),
    /// A boolean value.
    Boolean(bool),
    /// A string value.
    String(String),
    /// A list of string values.
    StringList(Vec<String>),
}

/// Represents a parsed line with a key and its associated value.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedLine {
    /// The key parsed from the line.
    pub key: String,
    /// The value associated with the key.
    pub value: OptionsDataType,
}

/// Error type for parsing failures.
#[derive(Debug, Clone)]
pub struct ParseError {
    /// Description of the parsing error.
    pub message: String,
}

impl fmt::Display for ParseError {
    /// Formats the error for display.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ParseError: {}", self.message)
    }
}

impl std::error::Error for ParseError {}

/// Parses a single line into a `ParsedLine`.
///
/// # Arguments
///
/// * `line` - The input string to parse.
///
/// # Returns
///
/// * `Ok(ParsedLine)` if parsing succeeds.
/// * `Err(ParseError)` if the line is malformed or the key is missing.
pub fn parse_line(line: &str) -> Result<ParsedLine, ParseError> {
    let (key, value_str) = split_line(line)?;

    if key.is_empty() {
        return Err(ParseError {
            message: "Key is missing or empty".to_string(),
        });
    }

    let value = parse_value(value_str);

    Ok(ParsedLine {
        key: key.to_string(),
        value,
    })
}

/// Splits a line into a key and value pair.
///
/// # Arguments
///
/// * `line` - The input string to split.
///
/// # Returns
///
/// * `Ok((&str, &str))` if the line is successfully split.
/// * `Err(ParseError)` if the line is malformed.
fn split_line(line: &str) -> Result<(&str, &str), ParseError> {
    let mut parts = line.splitn(2, ':');
    let key = parts.next().map(str::trim).unwrap_or("");
    let value_str = parts.next().map(str::trim).unwrap_or("");
    Ok((key, value_str))
}

/// Parses the value string into an `OptionsDataType`.
///
/// # Arguments
///
/// * `value_str` - The value string to parse.
///
/// # Returns
///
/// * `OptionsDataType` representing the parsed value.
fn parse_value(value_str: &str) -> OptionsDataType {
    match value_str {
        "" => OptionsDataType::String(String::new()),
        _ if value_str.parse::<i64>().is_ok() => OptionsDataType::Integer(value_str.parse().unwrap()),
        _ if value_str.parse::<f64>().is_ok() => OptionsDataType::Float(value_str.parse().unwrap()),
        "true" | "\"true\"" => OptionsDataType::Boolean(true),
        "false" | "\"false\"" => OptionsDataType::Boolean(false),
        _ if value_str.starts_with('[') && value_str.ends_with(']') => {
            let list_items = value_str[1..value_str.len() - 1]
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
            OptionsDataType::StringList(list_items)
        }
        _ => OptionsDataType::String(value_str.to_string()),
    }
}

/// Parses a multi-line string into a vector of `ParsedLine` objects.
///
/// Ignores empty lines and lines starting with `#` (comments).
///
/// # Arguments
///
/// * `content` - The multi-line string to parse.
///
/// # Returns
///
/// * `Vec<ParsedLine>` containing all successfully parsed lines.
pub fn parse_options_string(content: &str) -> Vec<ParsedLine> {
    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                None
            } else {
                parse_line(trimmed).ok()
            }
        })
        .collect()
}

/// Reads an options file and parses its contents into a vector of `ParsedLine` objects.
///
/// # Arguments
///
/// * `path` - The file path to read.
///
/// # Returns
///
/// * `Ok(Vec<ParsedLine>)` if the file is read and parsed successfully.
/// * `Err(std::io::Error)` if the file cannot be read.
pub fn parse_options_file(path: &str) -> Result<Vec<ParsedLine>, std::io::Error> {
    let content = std::fs::read_to_string(path)?;
    Ok(parse_options_string(&content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_integer_value() {
        let parsed = parse_line("count: 42").unwrap();
        assert_eq!(parsed.key, "count");
        match parsed.value {
            OptionsDataType::Integer(v) => assert_eq!(v, 42),
            _ => panic!("expected integer"),
        }
    }

    #[test]
    fn parses_float_value() {
        let parsed = parse_line("ratio: 3.14").unwrap();
        assert_eq!(parsed.key, "ratio");
        match parsed.value {
            OptionsDataType::Float(v) => assert_eq!(v, 3.14),
            _ => panic!("expected float"),
        }
    }

    #[test]
    fn parses_boolean_true_value() {
        let parsed = parse_line("enabled: true").unwrap();
        assert_eq!(parsed.key, "enabled");
        match parsed.value {
            OptionsDataType::Boolean(v) => assert!(v),
            _ => panic!("expected boolean true"),
        }
    }

    #[test]
    fn parses_boolean_false_value_with_quotes() {
        let parsed = parse_line("active:\"false\"").unwrap();
        assert_eq!(parsed.key, "active");
        match parsed.value {
            OptionsDataType::Boolean(v) => assert!(!v),
            _ => panic!("expected boolean false"),
        }
    }

    #[test]
    fn parses_string_list_value() {
        let parsed = parse_line("items: [apple, banana, cherry]").unwrap();
        assert_eq!(parsed.key, "items");
        match parsed.value {
            OptionsDataType::StringList(v) => assert_eq!(v, vec!["apple", "banana", "cherry"]),
            _ => panic!("expected string list"),
        }
    }

    #[test]
    fn parses_string_value_when_no_other_type_matches() {
        let parsed = parse_line("name: John Doe").unwrap();
        assert_eq!(parsed.key, "name");
        match parsed.value {
            OptionsDataType::String(v) => assert_eq!(v, "John Doe"),
            _ => panic!("expected string"),
        }
    }

    #[test]
    fn parses_empty_value_as_string() {
        let parsed = parse_line("empty:").unwrap();
        assert_eq!(parsed.key, "empty");
        match parsed.value {
            OptionsDataType::String(v) => assert_eq!(v, ""),
            _ => panic!("expected string"),
        }
    }
    #[test]
    fn trims_whitespace_around_key_and_value() {
        let parsed = parse_line("  key  :   value  ").unwrap();
        assert_eq!(parsed.key, "key");
        match parsed.value {
            OptionsDataType::String(v) => assert_eq!(v, "value"),
            _ => panic!("expected string"),
        }
    }

    #[test]
    fn parses_list_with_extra_spaces_and_empty_items() {
        let parsed = parse_line("list: [ a,  , b ,c ]").unwrap();
        assert_eq!(parsed.key, "list");
        match parsed.value {
            OptionsDataType::StringList(v) => assert_eq!(v, vec!["a", "", "b", "c"]),
            _ => panic!("expected string list"),
        }
    }

    #[test]
    fn parses_options_string_ignores_empty_and_comment_lines() {
        let content = r#"
        # This is a comment
        key1: 123

        key2: true
        # Another comment
        key3: [a, b, c]
    "#;
        let parsed = parse_options_string(content);
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0].key, "key1");
        assert_eq!(parsed[1].key, "key2");
        assert_eq!(parsed[2].key, "key3");
    }

    #[test]
    fn parses_options_string_handles_only_comments_and_empty_lines() {
        let content = r#"
        # comment
        # another comment

    "#;
        let parsed = parse_options_string(content);
        assert!(parsed.is_empty());
    }

    #[test]
    fn parses_options_string_handles_mixed_types() {
        let content = r#"
            int: 1
            float: 2.5
            bool: false
            str: hello
            list: [x, y, z]
        "#;
        let parsed = parse_options_string(content);
        assert_eq!(parsed.len(), 5);
        matches!(parsed[0].value, OptionsDataType::Integer(1));
        matches!(parsed[1].value, OptionsDataType::Float(f) if (f - 2.5).abs() < f64::EPSILON);
        matches!(parsed[2].value, OptionsDataType::Boolean(false));
        matches!(parsed[3].value, OptionsDataType::String(ref s) if s == "hello");
        matches!(parsed[4].value, OptionsDataType::StringList(ref v) if v == &["x", "y", "z"]);
    }

    #[test]
    fn parses_options_file_returns_error_for_nonexistent_file() {
        let result = parse_options_file("nonexistent_file.txt");
        assert!(result.is_err());
    }

    #[test]
    fn returns_error_for_missing_key() {
        let line = ": value";
        let result = parse_line(line);
        assert!(result.is_err());
        if let Err(e) = result {
            assert_eq!(e.message, "Key is missing or empty");
        }
    }


    #[test]
    fn parses_key_with_special_characters() {
        let parsed = parse_line("key-with-dash: value").unwrap();
        assert_eq!(parsed.key, "key-with-dash");
        match parsed.value {
            OptionsDataType::String(v) => assert_eq!(v, "value"),
            _ => panic!("expected string"),
        }
    }

    #[test]
    fn parses_value_with_colon() {
        let parsed = parse_line("key: value:with:colons").unwrap();
        assert_eq!(parsed.key, "key");
        match parsed.value {
            OptionsDataType::String(v) => assert_eq!(v, "value:with:colons"),
            _ => panic!("expected string"),
        }
    }

    #[test]
    fn parses_empty_key_with_non_empty_value() {
        let parsed = parse_line(": value").unwrap_err();
        assert_eq!(parsed.message, "Key is missing or empty");
    }

    #[test]
    fn parses_value_with_quotes() {
        let parsed = parse_line("key: \"quoted value\"").unwrap();
        assert_eq!(parsed.key, "key");
        match parsed.value {
            OptionsDataType::String(v) => assert_eq!(v, "\"quoted value\""),
            _ => panic!("expected string"),
        }
    }

    #[test]
    fn parses_list_with_nested_brackets() {
        let parsed = parse_line("key: [a, [b, c], d]").unwrap();
        assert_eq!(parsed.key, "key");
        match parsed.value {
            OptionsDataType::StringList(v) => assert_eq!(v, vec!["a", "[b", "c]", "d"]),
            _ => panic!("expected string list"),
        }
    }

    #[test]
    fn parses_key_with_leading_and_trailing_whitespace() {
        let parsed = parse_line("  key  : value").unwrap();
        assert_eq!(parsed.key, "key");
        match parsed.value {
            OptionsDataType::String(v) => assert_eq!(v, "value"),
            _ => panic!("expected string"),
        }
    }

    #[test]
    fn parses_value_with_leading_and_trailing_whitespace() {
        let parsed = parse_line("key:   value   ").unwrap();
        assert_eq!(parsed.key, "key");
        match parsed.value {
            OptionsDataType::String(v) => assert_eq!(v, "value"),
            _ => panic!("expected string"),
        }
    }

    #[test]
    fn parses_key_and_value_with_unicode_characters() {
        let parsed = parse_line("ключ: значение").unwrap();
        assert_eq!(parsed.key, "ключ");
        match parsed.value {
            OptionsDataType::String(v) => assert_eq!(v, "значение"),
            _ => panic!("expected string"),
        }
    }

    #[test]
    fn parses_value_with_escaped_characters() {
        let parsed = parse_line("key: value\\nwith\\tescapes").unwrap();
        assert_eq!(parsed.key, "key");
        match parsed.value {
            OptionsDataType::String(v) => assert_eq!(v, "value\\nwith\\tescapes"),
            _ => panic!("expected string"),
        }
    }

    #[test]
    fn parses_key_with_numbers() {
        let parsed = parse_line("key123: value").unwrap();
        assert_eq!(parsed.key, "key123");
        match parsed.value {
            OptionsDataType::String(v) => assert_eq!(v, "value"),
            _ => panic!("expected string"),
        }
    }
}