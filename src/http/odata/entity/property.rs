use crate::http::odata::edm::Type;

/// Internal structure for holding property values for an Entity
pub struct Property {
    name: String,
    ptype: Type,
}

impl Property {
    pub fn new(name: &str, ptype: Type) -> Self {
        Property {
            name: String::from(name),
            ptype: ptype,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn types(&self) -> Vec<&str> {
        self.ptype.ty()
    }

    pub fn format(&self) -> &str {
        self.ptype.format()
    }
}
