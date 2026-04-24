use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use sqlx::{Column, Row, TypeInfo};

/// Build a tagged value: `{ "type": "<pg_type>", "value": <val> }`.
fn tagged(pg_type: &str, value: serde_json::Value) -> serde_json::Value {
    serde_json::json!({ "type": pg_type, "value": value })
}

/// Build a tagged value for an unknown type: `{ "type": "unknown", "raw": "<text>" }`.
fn tagged_unknown(raw: &str) -> serde_json::Value {
    serde_json::json!({ "type": "unknown", "raw": raw })
}

/// Normalize a Postgres type name to a canonical frontend type string.
fn normalize_pg_type(type_name: &str) -> &str {
    match type_name {
        "BOOL" => "boolean",
        "INT2" => "smallint",
        "INT4" => "integer",
        "INT8" => "bigint",
        "FLOAT4" => "real",
        "FLOAT8" => "double",
        "NUMERIC" => "numeric",
        "MONEY" => "money",
        "TEXT" => "text",
        "VARCHAR" | "CHAR" | "BPCHAR" | "NAME" => "text",
        "BYTEA" => "bytea",
        "DATE" => "date",
        "TIME" | "TIMETZ" => "time",
        "TIMESTAMP" => "timestamp",
        "TIMESTAMPTZ" => "timestamptz",
        "INTERVAL" => "interval",
        "UUID" => "uuid",
        "JSON" => "json",
        "JSONB" => "jsonb",
        "XML" => "xml",
        "INET" => "inet",
        "CIDR" => "cidr",
        "MACADDR" | "MACADDR8" => "macaddr",
        "BIT" | "VARBIT" => "bit",
        "TSVECTOR" => "tsvector",
        "TSQUERY" => "tsquery",
        "POINT" | "LINE" | "LSEG" | "BOX" | "PATH" | "POLYGON" | "CIRCLE" => "geometry",
        "INT4RANGE" | "INT8RANGE" | "NUMRANGE" | "TSRANGE" | "TSTZRANGE" | "DATERANGE" => "range",
        "OID" => "integer",
        _ => {
            // Array types start with underscore in Postgres internal names
            if type_name.starts_with('_') {
                "array"
            } else {
                "unknown"
            }
        }
    }
}

/// Try to decode a column value as `Option<T>`, returning tagged JSON on success,
/// `Null` for SQL NULL, or a tagged-unknown fallback on decode error.
fn try_decode<T>(
    row: &sqlx::postgres::PgRow,
    idx: usize,
    canonical: &str,
    type_name: &str,
    f: impl FnOnce(T) -> serde_json::Value,
) -> serde_json::Value
where
    T: sqlx::Type<sqlx::Postgres> + for<'r> sqlx::Decode<'r, sqlx::Postgres>,
{
    row.try_get::<Option<T>, _>(idx).map_or_else(
        |_| tagged_unknown(type_name),
        |opt| opt.map_or(serde_json::Value::Null, |v| tagged(canonical, f(v))),
    )
}

/// Convert a PostgreSQL column value to a tagged JSON value.
/// Output format: `{ "type": "<pg_type>", "value": <json_val> }`.
/// NULL values are returned as `serde_json::Value::Null` (untagged).
pub(crate) fn pg_value_to_json(row: &sqlx::postgres::PgRow, idx: usize) -> serde_json::Value {
    let col = row.column(idx);
    let type_name = col.type_info().name();
    let canonical = normalize_pg_type(type_name);

    match type_name {
        "BOOL" => try_decode::<bool>(row, idx, canonical, type_name, serde_json::Value::Bool),
        "INT2" => try_decode::<i16>(row, idx, canonical, type_name, |v| serde_json::Value::Number(v.into())),
        "INT4" | "OID" => try_decode::<i32>(row, idx, canonical, type_name, |v| serde_json::Value::Number(v.into())),
        "INT8" => try_decode::<i64>(row, idx, canonical, type_name, |v| serde_json::Value::Number(v.into())),
        "FLOAT4" => try_decode::<f32>(row, idx, canonical, type_name, |v| {
            serde_json::Number::from_f64(f64::from(v))
                .map_or_else(|| serde_json::Value::String(v.to_string()), serde_json::Value::Number)
        }),
        "FLOAT8" => try_decode::<f64>(row, idx, canonical, type_name, |v| {
            serde_json::Number::from_f64(v)
                .map_or_else(|| serde_json::Value::String(v.to_string()), serde_json::Value::Number)
        }),
        "JSON" | "JSONB" => try_decode::<serde_json::Value>(row, idx, canonical, type_name, |v| v),
        "TIMESTAMP" => try_decode::<NaiveDateTime>(row, idx, canonical, type_name, |v| {
            serde_json::Value::String(v.format("%Y-%m-%d %H:%M:%S").to_string())
        }),
        "TIMESTAMPTZ" => try_decode::<chrono::DateTime<chrono::Utc>>(row, idx, canonical, type_name, |v| {
            serde_json::Value::String(v.format("%Y-%m-%dT%H:%M:%SZ").to_string())
        }),
        "DATE" => try_decode::<NaiveDate>(row, idx, canonical, type_name, |v| {
            serde_json::Value::String(v.format("%Y-%m-%d").to_string())
        }),
        "TIME" | "TIMETZ" => try_decode::<NaiveTime>(row, idx, canonical, type_name, |v| {
            serde_json::Value::String(v.format("%H:%M:%S").to_string())
        }),
        // For all other types, try as String — covers text, varchar,
        // uuid, inet, cidr, macaddr, interval, xml, numeric, money, bit, range, etc.
        _ => try_decode::<String>(row, idx, canonical, type_name, serde_json::Value::String),
    }
}
