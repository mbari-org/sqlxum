use chrono::{DateTime, NaiveDateTime, Utc};
use futures::StreamExt;
use serde_json::{json, Value};
use sqlx::postgres::{PgColumn, PgRow};
use sqlx::{Column, Row, TypeInfo};
use std::fmt::Display;
use std::time::Instant;

use crate::common::unescape_query;

/// Performs a query, returning a Json array with the result
pub async fn do_query(pool: &sqlx::PgPool, query: &str) -> anyhow::Result<Value> {
    let query = unescape_query(query);
    log::info!("do_query: {}", query);

    let mut result: Vec<Value> = vec![];

    let mut add_row = |row: &PgRow| {
        let mut obj = json!({});
        row.columns().iter().for_each(|col| {
            let value = get_col_value(row, col);
            obj[col.name()] = value;
        });
        result.push(obj);
    };

    let start = Instant::now();
    let mut stream = sqlx::query(&query).fetch(pool);
    while let Some(res) = stream.next().await {
        add_row(&(res?));
    }
    let elapsed = format!("{:?}", start.elapsed());

    Ok(json!({
        "query": query,
        "result": result,
        "elapsed": elapsed,
    }))
}

/// A very small set of types are handled, some in an ad hoc way.
/// Expand/refine to your heart's content.
fn get_col_value(row: &PgRow, col: &PgColumn) -> Value {
    let name = col.name();
    // Why matching on the (str) type name (and not on a nice enum) below?
    // See https://github.com/launchbadge/sqlx/issues/1369
    let type_str = col.type_info().name();
    match type_str {
        "INT2" => value_as_i64::<i16>(row, name),
        "INT4" => value_as_i64::<i32>(row, name),
        "INT8" => value_as_i64::<i64>(row, name),
        "FLOAT4" => value_as_f64::<f32>(row, name),
        "FLOAT8" => value_as_f64::<f64>(row, name),
        "VARCHAR" | "TEXT" => value_as_string::<&str>(row, name),
        "TIMESTAMPTZ" => value_as_string::<DateTime<Utc>>(row, name),
        "TIMESTAMP" => value_as_string::<NaiveDateTime>(row, name),
        "BOOL" => value_as_bool(row, name),
        "BYTEA" => bytea_value(row, name),
        "UUID" => value_as_string::<uuid::Uuid>(row, name),

        _ => Value::String(format!("(UNHANDLED TYPE: {})", type_str)),
    }
}

fn value_as_f64<'r, T>(row: &'r PgRow, name: &str) -> Value
where
    T: Into<f64> + sqlx::Decode<'r, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>,
{
    match row.try_get::<Option<T>, _>(name) {
        Ok(Some(n)) => {
            let f: f64 = n.into();
            json!(f)
        }
        Ok(None) => json!(null),
        Err(e) => {
            log::error!("{}", e);
            json!(null)
        }
    }
}

fn value_as_i64<'r, T>(row: &'r PgRow, name: &str) -> Value
where
    T: Into<i64> + sqlx::Decode<'r, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>,
{
    match row.try_get::<Option<T>, _>(name) {
        Ok(Some(n)) => {
            let i: i64 = n.into();
            json!(i)
        }
        Ok(None) => json!(null),
        Err(e) => {
            log::error!("{}", e);
            json!(null)
        }
    }
}

fn value_as_bool(row: &PgRow, name: &str) -> Value {
    match row.try_get::<Option<bool>, _>(name) {
        Ok(Some(val)) => json!(val),
        Ok(None) => json!(null),
        Err(e) => {
            log::error!("{}", e);
            json!(null)
        }
    }
}

/// String version of a column value
fn value_as_string<'r, T>(row: &'r PgRow, name: &str) -> Value
where
    T: Display + sqlx::Decode<'r, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>,
{
    match row.try_get::<Option<T>, _>(name) {
        Ok(Some(val)) => json!(format!("{}", val)),
        Ok(None) => json!(null),
        Err(e) => json!(format!("ERROR: {}", e)),
    }
}

/// Ad hoc convenience to get a `bytea` column as a string
fn bytea_value(row: &PgRow, name: &str) -> Value {
    match row.try_get::<Option<Vec<u8>>, _>(name) {
        Ok(Some(val)) => json!(bytea_as_string(&val)),
        Ok(None) => json!(null),
        Err(e) => {
            log::error!("{}", e);
            json!(null)
        }
    }
}

fn bytea_as_string(val: &[u8]) -> String {
    let len = val.len();
    let mut suffix_vec = "";
    let mut suffix_str = "";
    let val = if len <= 10 {
        val
    } else {
        suffix_vec = ", ...";
        suffix_str = "...";
        &val[..std::cmp::min(len, 12)]
    };
    let ascii = String::from_utf8_lossy(val).to_string();
    let elements = val
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    format!("({len}) [{elements}{suffix_vec}] -> ascii='{ascii}{suffix_str}'")
}
