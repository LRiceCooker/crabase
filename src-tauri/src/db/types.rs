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

/// Convert a PostgreSQL column value to a tagged JSON value.
/// Output format: `{ "type": "<pg_type>", "value": <json_val> }`.
/// NULL values are returned as `serde_json::Value::Null` (untagged).
pub(crate) fn pg_value_to_json(row: &sqlx::postgres::PgRow, idx: usize) -> serde_json::Value {
    let col = row.column(idx);
    let type_name = col.type_info().name();
    let canonical = normalize_pg_type(type_name);

    // Check for NULL first via a raw decode attempt
    match type_name {
        "BOOL" => match row.try_get::<Option<bool>, _>(idx) {
            Ok(Some(v)) => tagged(canonical, serde_json::Value::Bool(v)),
            Ok(None) => serde_json::Value::Null,
            Err(_) => tagged_unknown(type_name),
        },
        "INT2" => match row.try_get::<Option<i16>, _>(idx) {
            Ok(Some(v)) => tagged(canonical, serde_json::Value::Number(v.into())),
            Ok(None) => serde_json::Value::Null,
            Err(_) => tagged_unknown(type_name),
        },
        "INT4" | "OID" => match row.try_get::<Option<i32>, _>(idx) {
            Ok(Some(v)) => tagged(canonical, serde_json::Value::Number(v.into())),
            Ok(None) => serde_json::Value::Null,
            Err(_) => tagged_unknown(type_name),
        },
        "INT8" => match row.try_get::<Option<i64>, _>(idx) {
            Ok(Some(v)) => tagged(canonical, serde_json::Value::Number(v.into())),
            Ok(None) => serde_json::Value::Null,
            Err(_) => tagged_unknown(type_name),
        },
        "FLOAT4" => match row.try_get::<Option<f32>, _>(idx) {
            Ok(Some(v)) => {
                let n = serde_json::Number::from_f64(v as f64)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::String(v.to_string()));
                tagged(canonical, n)
            }
            Ok(None) => serde_json::Value::Null,
            Err(_) => tagged_unknown(type_name),
        },
        "FLOAT8" => match row.try_get::<Option<f64>, _>(idx) {
            Ok(Some(v)) => {
                let n = serde_json::Number::from_f64(v)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::String(v.to_string()));
                tagged(canonical, n)
            }
            Ok(None) => serde_json::Value::Null,
            Err(_) => tagged_unknown(type_name),
        },
        "JSON" | "JSONB" => match row.try_get::<Option<serde_json::Value>, _>(idx) {
            Ok(Some(v)) => tagged(canonical, v),
            Ok(None) => serde_json::Value::Null,
            Err(_) => tagged_unknown(type_name),
        },
        "TIMESTAMP" => match row.try_get::<Option<NaiveDateTime>, _>(idx) {
            Ok(Some(v)) => tagged(canonical, serde_json::Value::String(v.format("%Y-%m-%d %H:%M:%S").to_string())),
            Ok(None) => serde_json::Value::Null,
            Err(_) => tagged_unknown(type_name),
        },
        "TIMESTAMPTZ" => match row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>(idx) {
            Ok(Some(v)) => tagged(canonical, serde_json::Value::String(v.format("%Y-%m-%dT%H:%M:%SZ").to_string())),
            Ok(None) => serde_json::Value::Null,
            Err(_) => tagged_unknown(type_name),
        },
        "DATE" => match row.try_get::<Option<NaiveDate>, _>(idx) {
            Ok(Some(v)) => tagged(canonical, serde_json::Value::String(v.format("%Y-%m-%d").to_string())),
            Ok(None) => serde_json::Value::Null,
            Err(_) => tagged_unknown(type_name),
        },
        "TIME" | "TIMETZ" => match row.try_get::<Option<NaiveTime>, _>(idx) {
            Ok(Some(v)) => tagged(canonical, serde_json::Value::String(v.format("%H:%M:%S").to_string())),
            Ok(None) => serde_json::Value::Null,
            Err(_) => tagged_unknown(type_name),
        },
        _ => {
            // For all other types, try as String first — covers text, varchar,
            // uuid, inet, cidr, macaddr, interval, xml, numeric, money, bit, range, etc.
            match row.try_get::<Option<String>, _>(idx) {
                Ok(Some(v)) => tagged(canonical, serde_json::Value::String(v)),
                Ok(None) => serde_json::Value::Null,
                Err(_) => tagged_unknown(type_name),
            }
        }
    }
}
