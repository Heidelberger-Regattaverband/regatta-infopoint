/// Unquote a string by removing the leading and trailing single quotes.
/// # Arguments
/// * `string` - The string to unquote.
/// # Returns
/// The unquoted string.
pub(crate) fn unquote(string: &str) -> String {
    string.trim_start_matches('\'').trim_end_matches('\'').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unquote() {
        assert_eq!(unquote("'hello'"), "hello");
        assert_eq!(unquote("'world'"), "world");
    }
}
