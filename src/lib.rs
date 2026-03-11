use act_sdk::prelude::*;
use base64::Engine;
use rusqlite::{Connection, params_from_iter, types::Value};
use std::sync::Mutex;

static DB: Mutex<Option<Connection>> = Mutex::new(None);

fn get_or_open_db(path: &str) -> ActResult<()> {
    let mut guard = DB.lock().map_err(|e| ActError::internal(format!("Lock error: {e}")))?;
    if guard.is_none() {
        let conn = Connection::open(path)
            .map_err(|e| ActError::internal(format!("Cannot open database: {e}")))?;
        // Enable WAL mode for better concurrent access
        conn.execute_batch("PRAGMA journal_mode=WAL;")
            .map_err(|e| ActError::internal(format!("PRAGMA error: {e}")))?;
        *guard = Some(conn);
    }
    Ok(())
}

fn with_db<F, T>(path: &str, f: F) -> ActResult<T>
where
    F: FnOnce(&Connection) -> ActResult<T>,
{
    get_or_open_db(path)?;
    let guard = DB.lock().map_err(|e| ActError::internal(format!("Lock error: {e}")))?;
    f(guard.as_ref().unwrap())
}

#[derive(Deserialize, JsonSchema)]
struct Config {
    /// Path to SQLite database file
    database_path: String,
}

#[act_component(
    name = "sqlite",
    version = "0.1.0",
    description = "SQLite database operations",
)]
mod component {
    use super::*;

    /// Execute a SELECT query and return results as JSON.
    #[act_tool(description = "Execute a read-only SQL query (SELECT) and return results as JSON array", read_only)]
    fn query(
        #[doc = "SQL SELECT query to execute"] sql: String,
        #[doc = "Query parameters as JSON array (optional)"] params: Option<Vec<serde_json::Value>>,
        ctx: &mut ActContext<Config>,
    ) -> ActResult<String> {
        let path = ctx.config().database_path.clone();
        with_db(&path, |conn| {
            let param_values = json_params_to_sqlite(params.as_deref())?;
            let mut stmt = conn.prepare(&sql)
                .map_err(|e| ActError::invalid_args(format!("SQL error: {e}")))?;

            let column_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();

            let rows: Vec<serde_json::Value> = stmt.query_map(
                params_from_iter(param_values.iter()),
                |row| {
                    let mut obj = serde_json::Map::new();
                    for (i, name) in column_names.iter().enumerate() {
                        let val: Value = row.get(i)?;
                        obj.insert(name.clone(), sqlite_value_to_json(&val));
                    }
                    Ok(serde_json::Value::Object(obj))
                },
            )
            .map_err(|e| ActError::internal(format!("Query error: {e}")))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ActError::internal(format!("Row error: {e}")))?;

            serde_json::to_string_pretty(&rows)
                .map_err(|e| ActError::internal(format!("JSON error: {e}")))
        })
    }

    /// Execute a write SQL statement (INSERT, UPDATE, DELETE, CREATE, etc.)
    #[act_tool(description = "Execute a write SQL statement (INSERT, UPDATE, DELETE, CREATE TABLE, etc.)")]
    fn execute(
        #[doc = "SQL statement to execute"] sql: String,
        #[doc = "Statement parameters as JSON array (optional)"] params: Option<Vec<serde_json::Value>>,
        ctx: &mut ActContext<Config>,
    ) -> ActResult<String> {
        let path = ctx.config().database_path.clone();
        with_db(&path, |conn| {
            let param_values = json_params_to_sqlite(params.as_deref())?;
            let affected = conn.execute(&sql, params_from_iter(param_values.iter()))
                .map_err(|e| ActError::invalid_args(format!("SQL error: {e}")))?;
            Ok(serde_json::json!({
                "rows_affected": affected,
                "last_insert_rowid": conn.last_insert_rowid(),
            }).to_string())
        })
    }

    /// List all tables in the database.
    #[act_tool(description = "List all tables in the SQLite database", read_only)]
    fn list_tables(ctx: &mut ActContext<Config>) -> ActResult<String> {
        let path = ctx.config().database_path.clone();
        with_db(&path, |conn| {
            let mut stmt = conn.prepare(
                "SELECT name, type FROM sqlite_master WHERE type IN ('table', 'view') AND name NOT LIKE 'sqlite_%' ORDER BY name"
            ).map_err(|e| ActError::internal(format!("SQL error: {e}")))?;

            let tables: Vec<serde_json::Value> = stmt.query_map([], |row| {
                let name: String = row.get(0)?;
                let typ: String = row.get(1)?;
                Ok(serde_json::json!({"name": name, "type": typ}))
            })
            .map_err(|e| ActError::internal(format!("Query error: {e}")))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ActError::internal(format!("Row error: {e}")))?;

            serde_json::to_string_pretty(&tables)
                .map_err(|e| ActError::internal(format!("JSON error: {e}")))
        })
    }

    /// Get detailed schema for a specific table.
    #[act_tool(description = "Get column names, types, and constraints for a table", read_only)]
    fn describe_table(
        #[doc = "Table name to describe"] table: String,
        ctx: &mut ActContext<Config>,
    ) -> ActResult<String> {
        let path = ctx.config().database_path.clone();
        with_db(&path, |conn| {
            // Validate table name exists
            let exists: bool = conn.query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE name = ?1 AND type IN ('table', 'view')",
                [&table],
                |row| row.get(0),
            ).map_err(|e| ActError::internal(format!("SQL error: {e}")))?;

            if !exists {
                return Err(ActError::not_found(format!("Table not found: {table}")));
            }

            let mut stmt = conn.prepare(&format!("PRAGMA table_info('{}')", table.replace('\'', "''")))
                .map_err(|e| ActError::internal(format!("SQL error: {e}")))?;

            let columns: Vec<serde_json::Value> = stmt.query_map([], |row| {
                let cid: i64 = row.get(0)?;
                let name: String = row.get(1)?;
                let col_type: String = row.get(2)?;
                let notnull: bool = row.get(3)?;
                let default: Value = row.get(4)?;
                let pk: bool = row.get(5)?;
                Ok(serde_json::json!({
                    "cid": cid,
                    "name": name,
                    "type": col_type,
                    "notnull": notnull,
                    "default": sqlite_value_to_json(&default),
                    "primary_key": pk,
                }))
            })
            .map_err(|e| ActError::internal(format!("Query error: {e}")))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ActError::internal(format!("Row error: {e}")))?;

            // Also get CREATE TABLE statement
            let create_sql: String = conn.query_row(
                "SELECT sql FROM sqlite_master WHERE name = ?1",
                [&table],
                |row| row.get(0),
            ).unwrap_or_default();

            serde_json::to_string_pretty(&serde_json::json!({
                "table": table,
                "columns": columns,
                "create_sql": create_sql,
            }))
            .map_err(|e| ActError::internal(format!("JSON error: {e}")))
        })
    }

    /// Execute multiple SQL statements in a transaction.
    #[act_tool(description = "Execute multiple SQL statements in a single transaction")]
    fn execute_batch(
        #[doc = "SQL statements separated by semicolons"] sql: String,
        ctx: &mut ActContext<Config>,
    ) -> ActResult<String> {
        let path = ctx.config().database_path.clone();
        with_db(&path, |conn| {
            conn.execute_batch(&sql)
                .map_err(|e| ActError::invalid_args(format!("SQL error: {e}")))?;
            Ok(r#"{"status": "ok"}"#.to_string())
        })
    }
}

fn json_params_to_sqlite(params: Option<&[serde_json::Value]>) -> ActResult<Vec<Value>> {
    let Some(params) = params else { return Ok(vec![]); };
    params.iter().map(|v| match v {
        serde_json::Value::Null => Ok(Value::Null),
        serde_json::Value::Bool(b) => Ok(Value::Integer(if *b { 1 } else { 0 })),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() { Ok(Value::Integer(i)) }
            else if let Some(f) = n.as_f64() { Ok(Value::Real(f)) }
            else { Err(ActError::invalid_args("Unsupported number type")) }
        }
        serde_json::Value::String(s) => Ok(Value::Text(s.clone())),
        _ => Err(ActError::invalid_args("Only scalar values supported as params")),
    }).collect()
}

fn sqlite_value_to_json(val: &Value) -> serde_json::Value {
    match val {
        Value::Null => serde_json::Value::Null,
        Value::Integer(i) => serde_json::json!(i),
        Value::Real(f) => serde_json::json!(f),
        Value::Text(s) => serde_json::json!(s),
        Value::Blob(b) => serde_json::json!(base64::engine::general_purpose::STANDARD.encode(b)),
    }
}
