#[must_use]
pub fn quote_ident(ident: &str) -> String {
    format!("\"{}\"", ident.replace('"', "\"\""))
}

#[must_use]
pub fn quote_literal(string: &str) -> String {
    format!("'{}'", string.replace('\'', "''"))
}

#[cfg(test)]
mod tests {
    use super::{quote_ident, quote_literal};

    #[test]
    fn quote_ident_wraps_in_double_quotes() {
        assert_eq!(quote_ident("users"), "\"users\"");
    }

    #[test]
    fn quote_ident_doubles_inner_quotes() {
        assert_eq!(quote_ident("we\"ird"), "\"we\"\"ird\"");
    }

    #[test]
    fn quote_ident_neutralizes_statement_separator() {
        // Everything after the name stays inside the identifier and cannot
        // start a new statement.
        assert_eq!(
            quote_ident("sp1; DROP TABLE users"),
            "\"sp1; DROP TABLE users\""
        );
    }

    #[test]
    fn quote_literal_doubles_single_quotes() {
        assert_eq!(quote_literal("O'Reilly"), "'O''Reilly'");
    }
}
