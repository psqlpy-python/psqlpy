#[must_use]
pub fn quote_ident(ident: &str) -> String {
    format!("\"{}\"", ident.replace('"', "\"\""))
}

#[must_use]
pub fn quote_literal(string: &str) -> String {
    format!("'{}'", string.replace('\'', "''"))
}
