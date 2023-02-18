use super::error::Error;
use actix_web_lab::__reexports::serde_json;
use serde_json::Value;

/// Used for routing requests and for simplifying CRUD-Q implementation. Providing the
/// correct HTTP StatusCode is handled by the server.
pub enum Res {
    Succ(Option<Value>),
    Created(Value),
    Err(Error),
}
