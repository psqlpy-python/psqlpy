use postgres_types::Type;

pub fn array_type_to_single_type(array_type: &Type) -> Type {
    match *array_type {
        Type::BOOL_ARRAY => Type::BOOL,
        Type::UUID_ARRAY => Type::UUID_ARRAY,
        Type::VARCHAR_ARRAY => Type::VARCHAR,
        Type::TEXT_ARRAY => Type::TEXT,
        Type::INT2_ARRAY => Type::INT2,
        Type::INT4_ARRAY => Type::INT4,
        Type::INT8_ARRAY => Type::INT8,
        Type::FLOAT4_ARRAY => Type::FLOAT4,
        Type::FLOAT8_ARRAY => Type::FLOAT8,
        Type::MONEY_ARRAY => Type::MONEY,
        Type::INET_ARRAY => Type::INET,
        Type::JSON_ARRAY => Type::JSON,
        Type::JSONB_ARRAY => Type::JSONB,
        Type::DATE_ARRAY => Type::DATE,
        Type::TIME_ARRAY => Type::TIME,
        Type::TIMESTAMP_ARRAY => Type::TIMESTAMP,
        Type::TIMESTAMPTZ_ARRAY => Type::TIMESTAMPTZ,
        Type::INTERVAL_ARRAY => Type::INTERVAL,
        Type::MACADDR_ARRAY => Type::MACADDR,
        Type::MACADDR8_ARRAY => Type::MACADDR8,
        Type::POINT_ARRAY => Type::POINT,
        Type::BOX_ARRAY => Type::BOX,
        Type::PATH_ARRAY => Type::PATH,
        Type::LINE_ARRAY => Type::LINE,
        Type::LSEG_ARRAY => Type::LSEG,
        Type::CIRCLE_ARRAY => Type::CIRCLE,
        _ => Type::ANY,
    }
}
