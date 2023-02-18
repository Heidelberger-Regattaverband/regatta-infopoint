use super::{entity::EntityDescr, property::Property};
use crate::http::odata::service::{error::Error, handler::Res};
use actix_web_lab::__reexports::serde_json;
use serde_json::Value;

/// Internal descriptor for an EntitySet for usage by the Model. Provides access
/// to the underlying desciptor for the EntityType.
pub trait EntitySetDescr {
    fn name(&self) -> String;
    fn descriptor(&self) -> EntityDescr;
}

/// Trait for declaring CRUD-Q implementation.
pub trait EntitySet {
    fn create(&self, _value: Value) -> Res {
        Res::Err(Error::NoImpl)
    }

    fn read(&self, _key: String) -> Res {
        Res::Err(Error::NoImpl)
    }

    fn read_list(&self) -> Res {
        Res::Succ(Some(serde_json::json!(null)))
    }

    fn update(&self, _json: Value) -> Res {
        Res::Err(Error::NoImpl)
    }

    fn delete(&self, _key: Property) -> Res {
        Res::Err(Error::NoImpl)
    }

    fn query(&self, _params: Vec<String>) -> Res {
        Res::Err(Error::NoImpl)
    }
}
