/// An Edm type
pub enum Type {
    Binary,
    Boolean,
    Byte,
    Decimal,
    Double,
    Int16,
    Int32,
    Int64,
    String,
    //Binary([u8]),
    //DateTime:
    //DateTimeOffset:
    //Guid: Guid
    //SByte:,
    //Single:
    //Time: Timespan
}

impl Type {
    /// Matches a string to the Edm type to which it references
    pub fn from(s: &str) -> Type {
        match s {
            "Binary" => Type::Binary,
            "Boolean" => Type::Boolean,
            "Byte" => Type::Byte,
            "Decimal" => Type::Decimal,
            "Double" => Type::Double,
            "Int16" => Type::Int16,
            "Int32" => Type::Int32,
            "Int64" => Type::Int64,
            "String" => Type::String,
            _ => panic!("Unable to parse invalid Edm::Type!"),
        }
    }

    /// Returns the underlying types for a given Edm type (I know... see standard!)
    pub fn ty(&self) -> Vec<&str> {
        match self {
            &Type::Binary => vec!["string"],
            &Type::Boolean => vec!["boolean"],
            &Type::Byte => vec!["integer"],
            &Type::Decimal => vec!["number", "string"],
            &Type::Double => vec!["number", "string"],
            &Type::Int16 => vec!["integer"],
            &Type::Int32 => vec!["integer"],
            &Type::Int64 => vec!["integer"],
            &Type::String => vec!["string"],
        }
    }

    /// Returns the underlying format for a given Edm type (I know... see standard!)
    pub fn format(&self) -> &str {
        match self {
            &Type::Binary => "base64url",
            &Type::Boolean => "",
            &Type::Byte => "uint8",
            &Type::Decimal => "decimal",
            &Type::Double => "double",
            &Type::Int16 => "int16",
            &Type::Int32 => "int32",
            &Type::Int64 => "int64",
            &Type::String => "",
        }
    }
}
