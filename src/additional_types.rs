use byteorder::{BigEndian, ReadBytesExt};
use geo_types::{coord, Coord, CoordFloat, CoordNum, Line, LineString, Polygon};
use itertools::Itertools;
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

build_additional_rust_type!(RustLine, Line);
build_additional_rust_type!(RustPolygon, Polygon);

impl<'a> FromSql<'a> for RustLine {
    fn from_sql(
        _ty: &Type,
        raw: &'a [u8],
    ) -> Result<RustLine, Box<dyn std::error::Error + Sync + Send>> {
        if raw.len() == 4 {
            let mut vec_raw = vec![];
            vec_raw.extend_from_slice(raw);
            let mut buf = vec_raw.as_slice();

            let x1 = buf.read_f64::<BigEndian>()?;
            let y1 = buf.read_f64::<BigEndian>()?;
            let first_coord = coord!(x: x1, y: y1);

            let x2 = buf.read_f64::<BigEndian>()?;
            let y2 = buf.read_f64::<BigEndian>()?;
            let second_coord = coord!(x: x2, y: y2);

            let new_line = Line::new(first_coord, second_coord);
            return Ok(RustLine::new(new_line));
        }
        Err("Cannot convert PostgreSQL LINE into rust Line".into())
    }

    fn accepts(_ty: &Type) -> bool {
        true
    }
}

impl<'a> FromSql<'a> for RustPolygon {
    fn from_sql(
        _ty: &Type,
        raw: &'a [u8],
    ) -> Result<RustPolygon, Box<dyn std::error::Error + Sync + Send>> {
        if raw.len() % 2 == 0 {
            let mut vec_raw = vec![];
            vec_raw.extend_from_slice(raw);
            let mut buf = vec_raw.as_slice();

            let mut vec_raw_coord = vec![];
            buf.read_f64_into::<BigEndian>(&mut vec_raw_coord);

            let mut vec_coord = vec![];
            for (x1, y1) in vec_raw_coord.into_iter().tuples() {
                vec_coord.push(coord!(x: x1, y: y1));
            }

            let polygon_exterior = LineString::new(vec_coord);
            let new_polygon = Polygon::new(polygon_exterior, vec![]);
            return Ok(RustPolygon::new(new_polygon));
        }
        Err("Cannot convert PostgreSQL POLYGON into rust Polygon".into())
    }

    fn accepts(_ty: &Type) -> bool {
        true
    }
}

// add macro for creating circles

#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Circle<T: CoordNum = f64> {
    center: Coord<T>,
    radius: T,
}

impl<T: CoordNum> Circle<T> {
    pub fn new(x: T, y: T, r: T) -> Self {
        Self {
            center: coord!(x: x, y: y),
            radius: r,
        }
    }

    pub fn center(self) -> Coord<T> {
        self.center
    }

    pub fn set_center(&mut self, center: Coord<T>) -> &mut Self {
        self.center = center;
        self
    }

    pub fn center_mut(&mut self) -> &mut Coord<T> {
        &mut self.center
    }

    pub fn radius(self) -> T {
        self.radius
    }

    pub fn set_radius(&mut self, radius: T) -> &mut Self {
        self.radius = radius;
        self
    }

    pub fn radius_mut(&mut self) -> &mut T {
        &mut self.radius
    }
}

impl<T: CoordFloat> Circle<T> {
    pub fn distance_from_center_to(self, point: &Coord<T>) -> T {
        let dx = self.center.x - point.x;
        let dy = self.center.y - point.y;
        dx.hypot(dy)
    }

    pub fn contains(self, point: &Coord<T>) -> bool {
        self.distance_from_center_to(&point) <= self.radius
    }

    pub fn intersects(self, other: &Self) -> bool {
        self.distance_from_center_to(&other.center) <= self.radius + other.radius
    }
}

impl<T: CoordNum> Default for Circle<T> {
    fn default() -> Self {
        Self {
            center: coord! {x: T::zero(), y: T::zero()},
            radius: T::zero(),
        }
    }
}

impl<'a> FromSql<'a> for Circle {
    fn from_sql(
        _ty: &Type,
        raw: &'a [u8],
    ) -> Result<Circle, Box<dyn std::error::Error + Sync + Send>> {
        if raw.len() == 3 {
            let mut vec_raw = vec![];
            vec_raw.extend_from_slice(raw);
            let mut buf = vec_raw.as_slice();

            let x = buf.read_f64::<BigEndian>()?;
            let y = buf.read_f64::<BigEndian>()?;
            let r = buf.read_f64::<BigEndian>()?;

            let new_circle = Circle::new(x, y, r);
            return Ok(new_circle);
        }
        Err("Cannot convert PostgreSQL CIRCLE into rust Circle".into())
    }

    fn accepts(_ty: &Type) -> bool {
        true
    }
}
