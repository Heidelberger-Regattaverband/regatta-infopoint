/// Unquote a string by removing the leading and trailing single quotes.
/// # Arguments
/// * `string` - The string to unquote.
/// # Returns
/// The unquoted string.
pub(crate) fn unquote(string: &str) -> String {
    string.trim_start_matches('\'').trim_end_matches('\'').to_string()
}

/// Print a string with whitespaces replaced by escape sequences.
/// # Arguments
/// * `str` - The string to print.
/// # Returns
/// The string with whitespaces replaced by escape sequences.
pub(crate) fn print_whitespaces(str: &str) -> String {
    str.replace("\r", "\\r").replace("\n", "\\n").replace("\t", "\\t")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unquote() {
        assert_eq!(unquote("'hello'"), "hello");
        assert_eq!(unquote("'world'"), "world");
    }

    #[test]
    fn test_print_whitespaces() {
        assert_eq!(print_whitespaces("hello\nworld"), "hello\\nworld");
        assert_eq!(print_whitespaces("hello\tworld"), "hello\\tworld");
    }
}
