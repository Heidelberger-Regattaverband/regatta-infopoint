use super::property::Property;

/// Internal type holding metadata for EntityTypes
pub struct EntityDescr {
    pub name: String,
    pub keys: Vec<String>,
    pub properties: Vec<Property>,
}

impl EntityDescr {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn keys(&self) -> &[String] {
        &self.keys
    }

    pub fn properties(&self) -> &[Property] {
        &self.properties
    }
}

pub trait Entity {
    /// Used to expose fields to model. Passed-up to Model through EntitySet
    fn describe() -> EntityDescr;
}
