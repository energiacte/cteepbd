use std::fmt;

#[derive(Debug)]
pub enum EpbdError {
    NumberParseError(String),
    MetaParseError(String),
    ComponentParseError(String),
    ComponentsParseError(String),
    WFactorParseError(String),
    RenNrenCo2ParseError(String),
    CarrierUnknown(String),
    CTypeUnkwown(String),
    CSubtypeUnknown(String),
    ServiceUnknown(String),
    SourceUnknown(String),
    DestUnknown(String),
    StepUnknown(String),
    Area(String),
    Location(String),
    FactorNotFound(String),
}

impl fmt::Display for EpbdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use EpbdError::*;
        match self {
            NumberParseError(num) => write!(f, "Could not parse number value: {}", num),
            MetaParseError(input) => write!(f, "Could not parse Meta from \"{}\"", input),
            ComponentParseError(input) => write!(f, "Could not parse Component from \"{}\"", input),
            ComponentsParseError(input) => write!(f, "Could not parse Components from \"{}\"", input),
            WFactorParseError(input) => write!(f, "Could not parse Factor from \"{}\"", input),
            RenNrenCo2ParseError(input) => write!(f, "Could not parse RenNrenCo2 from \"{}\"", input),
            CarrierUnknown(input) => write!(f, "Unknown Carrier: \"{}\"", input),
            CTypeUnkwown(input) => write!(f, "Unknown Ctype: \"{}\"", input),
            CSubtypeUnknown(input) => write!(f, "Unknown CSubtype: \"{}\"", input),
            ServiceUnknown(input) => write!(f, "Unknown Service: \"{}\"", input),
            SourceUnknown(input) => write!(f, "Unknown Source: \"{}\"", input),
            DestUnknown(input) => write!(f, "Unknown Dest: \"{}\"", input),
            StepUnknown(input) => write!(f, "Unknown Step: \"{}\"", input),
            Area(area) => write!(f, "Unexpected reference area value: {}", area),
            Location(loc) => write!(f, "Unknown location; \"{}\"", loc),
            FactorNotFound(desc) => write!(f, "Conversion factor not found: {}", desc),
        }
    }
}

impl std::error::Error for EpbdError {}

pub type Result<T> = std::result::Result<T, EpbdError>;

impl From<std::num::ParseFloatError> for EpbdError {
    fn from(err: std::num::ParseFloatError) -> Self {
        EpbdError::NumberParseError(err.to_string())
    }
}
