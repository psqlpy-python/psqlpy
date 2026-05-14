use postgres_types::FromSql;
use pyo3::{types::PyString, Bound, IntoPyObject, Python};
use tokio_postgres::types::Type;

use crate::exceptions::rust_errors::RustPSQLDriverError;

/// Wrapper around the single-byte payload of `PostgreSQL`'s internal `"char"`
/// type (OID 18, distinct from `character(n)`/BPCHAR). Bytes 0..=255 map to
/// Unicode code points 0..=255 (Latin-1 round-trip), matching psycopg2/psycopg3.
#[derive(Clone, Copy)]
pub struct InternalChar(u8);

impl<'py> IntoPyObject<'py> for InternalChar {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = RustPSQLDriverError;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let mut tmp = [0u8; 4];
        let s = char::from(self.0).encode_utf8(&mut tmp);
        Ok(PyString::new(py, s))
    }
}

impl<'a> FromSql<'a> for InternalChar {
    fn from_sql(
        _ty: &Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        // The `"char"` binary wire format is exactly one byte. Read it as `u8`
        // directly — the `i8`-then-cast route through tokio_postgres' `FromSql`
        // impl trips clippy::cast_sign_loss in pedantic mode for no gain.
        let [byte] = *raw else {
            return Err(format!("\"char\" expected 1 byte, got {}", raw.len()).into());
        };
        Ok(InternalChar(byte))
    }

    fn accepts(ty: &Type) -> bool {
        *ty == Type::CHAR
    }
}

#[cfg(test)]
impl InternalChar {
    pub(crate) fn byte(self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::InternalChar;
    use postgres_types::{FromSql, Type};

    #[test]
    fn from_sql_round_trips_full_byte_range() {
        // The signed-byte cast (i8 -> u8) inside from_sql must preserve every
        // raw byte. Cover all 256 values so a sign-extension or normalization
        // regression cannot slip through.
        for b in 0u16..=255 {
            let byte = b as u8;
            let buf = [byte];
            let decoded =
                <InternalChar as FromSql>::from_sql(&Type::CHAR, &buf).expect("char decode");
            assert_eq!(decoded.byte(), byte, "byte 0x{byte:02x} not preserved");
        }
    }

    #[test]
    fn accepts_only_char_type() {
        assert!(<InternalChar as FromSql>::accepts(&Type::CHAR));
        assert!(!<InternalChar as FromSql>::accepts(&Type::TEXT));
        assert!(!<InternalChar as FromSql>::accepts(&Type::VARCHAR));
        assert!(!<InternalChar as FromSql>::accepts(&Type::BPCHAR));
    }
}
