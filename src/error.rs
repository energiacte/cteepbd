use std::fmt;

#[derive(Debug)]
pub enum EpbdError {
    Parse {
        from: String,
        into: String,
        desc: &'static str,
    },
}

impl fmt::Display for EpbdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EpbdError::Parse {
                from, into, desc, ..
            } => write!(f, "Could not parse {} from \"{}\" ({})", into, from, desc),
        }
    }
}

impl std::error::Error for EpbdError {

}

// pub type Result<T> = std::result::Result<T, Error>;

impl From<std::num::ParseFloatError> for EpbdError {
    fn from(err: std::num::ParseFloatError) -> Self {
        EpbdError::Parse {
            from: err.to_string(),
            into: "Number".into(),
            desc: "wrong number format",
        }
    }
}
