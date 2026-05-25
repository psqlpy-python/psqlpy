use postgres_types::{FromSql, Type};

/// Newtype that captures the raw column buffer bytes as `Option<&[u8]>`.
///
/// Replaces the fork-only `Row::col_buffer(i)` accessor: extracting a `RawBuf`
/// via `row.try_get::<usize, RawBuf>(i)` yields the same raw NULL-aware byte
/// slice that the fork's `col_buffer` returned.
pub struct RawBuf<'a>(pub Option<&'a [u8]>);

impl<'a> FromSql<'a> for RawBuf<'a> {
    fn from_sql(
        _ty: &Type,
        raw: &'a [u8],
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        Ok(RawBuf(Some(raw)))
    }

    fn from_sql_null(_ty: &Type) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        Ok(RawBuf(None))
    }

    fn from_sql_nullable(
        _ty: &Type,
        raw: Option<&'a [u8]>,
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        Ok(RawBuf(raw))
    }

    fn accepts(_ty: &Type) -> bool {
        true
    }
}
