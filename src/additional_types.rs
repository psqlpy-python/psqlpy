use macaddr::MacAddr6;
use tokio_postgres::types::{FromSql, Type};

#[derive(Debug)]
pub struct RustMacAddr6 {
    inner: MacAddr6,
}

impl RustMacAddr6 {
    #[must_use]
    pub fn new(inner: MacAddr6) -> Self {
        RustMacAddr6 { inner }
    }

    #[must_use]
    pub fn inner(&self) -> &MacAddr6 {
        &self.inner
    }
}

impl<'a> FromSql<'a> for RustMacAddr6 {
    fn from_sql(
        _ty: &Type,
        raw: &'a [u8],
    ) -> Result<RustMacAddr6, Box<dyn std::error::Error + Sync + Send>> {
        if raw.len() == 6 {
            let new_mac_address = MacAddr6::new(raw[0], raw[1], raw[2], raw[3], raw[4], raw[5]);
            return Ok(RustMacAddr6::new(new_mac_address));
        }
        Err("Cannot convert PostgreSQL MACADDR into rust MacAddr6".into())
    }

    fn accepts(_ty: &Type) -> bool {
        true
    }
}
