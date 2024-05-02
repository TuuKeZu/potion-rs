use std::fmt::{self, Display};

use warp::reject::{self, Rejection};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/**
    ### Enum for representing the most common html errorcodes
    Implements following
    - `std::error::Error`

    Can be converted into following
    - `potion::Error`

    Using methods `.new()` or `.default()`

    ### Example usage
    ```
        fn test() -> Result<(), potion::Error> {
            return Err(HtmlError::Unauthorized.default())
        }

        fn test2() -> Result<(), potion::Error> {
            return Err(HtmlError::Unauthorized.new("Invalid password"))
        }
    ```
 */
pub enum HtmlError {
    Unauthorized,
    InvalidRequest,
    InternalServerError
}

impl HtmlError {
    /**
     * Convert into `potion::Error` with default information
     */
    pub fn default(self) -> Error {
        match self {
            HtmlError::Unauthorized => Error::new(401, "Invalid credentials"),
            HtmlError::InvalidRequest => Error::new(400, "Invalid request"),
            HtmlError::InternalServerError => Error::new(500, "Internal server error"),
        }
    }

    /**
     * Convert into `potion::Error` with information
     */
    pub fn new(self, info: &str) -> Error {
        match self {
            HtmlError::Unauthorized => Error::new(401, info),
            HtmlError::InvalidRequest => Error::new(400, info),
            HtmlError::InternalServerError => Error::new(500, info),
        }
    }
}

impl Display for HtmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}: {:?})", self, self.default())
    }
}

impl std::error::Error for HtmlError {}


#[derive(Debug)]
pub struct Error {
    pub code: i16,
    pub info: Option<String>
}

impl Error {
    pub fn new(code: i16, info: &str) -> Self {
        Self {
            code,
            info: Some(info.to_string())
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}: {:?})", self.code, &self.info)
    }
}

impl reject::Reject for Error {}
impl std::error::Error for Error {}

#[derive(Debug)]
pub struct TypeError {
    info: String
}

impl TypeError {
    pub fn new(info: &str) -> Self {
        Self {
            info: info.to_string()
        }
    }
}

impl Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({})", self.info)
    }
}

impl std::error::Error for TypeError {}
impl Into<Rejection> for TypeError {
    fn into(self) -> Rejection {
        HtmlError::InvalidRequest.new(&self.info).into()
    }
}