use macaddr::{MacAddr6, MacAddr8};
use tokio_postgres::types::{FromSql, Type};

macro_rules! build_additional_rust_type {
    ($st_name:ident, $rust_type:ty) => {
        #[derive(Debug)]
        pub struct $st_name {
            inner: $rust_type,
        }

        impl $st_name {
            #[must_use]
            pub fn new(inner: $rust_type) -> Self {
                $st_name { inner }
            }

            #[must_use]
            pub fn inner(&self) -> &$rust_type {
                &self.inner
            }
        }
    };
}

build_additional_rust_type!(RustMacAddr6, MacAddr6);
build_additional_rust_type!(RustMacAddr8, MacAddr8);

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

impl<'a> FromSql<'a> for RustMacAddr8 {
    fn from_sql(
        _ty: &Type,
        raw: &'a [u8],
    ) -> Result<RustMacAddr8, Box<dyn std::error::Error + Sync + Send>> {
        if raw.len() == 8 {
            let new_mac_address = MacAddr8::new(
                raw[0], raw[1], raw[2], raw[3], raw[4], raw[5], raw[6], raw[7],
            );
            return Ok(RustMacAddr8::new(new_mac_address));
        }
        Err("Cannot convert PostgreSQL MACADDR8 into rust MacAddr8".into())
    }

    fn accepts(_ty: &Type) -> bool {
        true
    }
}
