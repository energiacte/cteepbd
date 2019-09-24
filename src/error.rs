use std::fmt;

#[derive(Debug)]
pub enum EpbdError {
    Parse {
        from: String,
        into: String,
        desc: &'static str,
    },
    Area(String),
    Location(String),
    FactorNotFound(String),
}

impl fmt::Display for EpbdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use EpbdError::*;
        match self {
            Parse {
                from, into, desc, ..
            } => write!(f, "Could not parse {} from \"{}\" ({})", into, from, desc),
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
        EpbdError::Parse {
            from: err.to_string(),
            into: "Number".into(),
            desc: "wrong number format",
        }
    }
}
