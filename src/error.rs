use std::fmt::{self, Display};

use warp::reject::Rejection;

pub const ERROR_CODE_INFO: &[(i16, &str)] = &[
    (400, "Invalid request; The request failed to contain or contained invalid payload. This shouldn't happen with normal use, so take your time to report this issue if you did not modify request parameters by hand.\
     The server responded with the following information about the issue: "),
    (401, "Invalid credentials; The browser should redirect you in a second.... if it does,'t this is a bug! Please report this below: "),
    (403, "Permission denied; You were not allowed to perform this action. Unless you were trying to do something you are not allowed to, you should report this below: "),
    (500, "Internal server error, this error was automatically reported to our system. The server responded with the following information about the issue: ")
];

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
    InvalidSession,
    InternalServerError,
}

impl HtmlError {
    /**
     * Convert into `potion::Error` with default information
     */
    pub fn default(self) -> Error {
        match self {
            HtmlError::InvalidSession => Error::new(401, "Invalid credentials", None),
            HtmlError::Unauthorized => Error::new(403, "Permission denied", None),
            HtmlError::InvalidRequest => Error::new(400, "Invalid request", None),
            HtmlError::InternalServerError => Error::new(500, "Internal server error", None),
        }
    }

    /**
     * Convert into `potion::Error` with information
     */
    pub fn new(self, info: &str) -> Error {
        match self {
            Self::InvalidSession => Error::new(401, info, None),
            Self::Unauthorized => Error::new(403, info, None),
            Self::InvalidRequest => Error::new(400, info, None),
            Self::InternalServerError => Error::new(500, info, None),
        }
    }

    /**
     * Convert into `potion::Error` with information
     */
    pub fn redirect(self, info: &str, redirect: &str) -> Error {
        match self {
            HtmlError::Unauthorized => Error::new(401, info, Some(redirect.to_string())),
            HtmlError::InvalidSession => Error::new(403, info, Some(redirect.to_string())),
            HtmlError::InvalidRequest => Error::new(400, info, Some(redirect.to_string())),
            HtmlError::InternalServerError => Error::new(500, info, Some(redirect.to_string())),
        }
    }
}

impl Display for HtmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}: {:?})", self, self.default())
    }
}

impl std::error::Error for HtmlError {}

#[derive(Debug, Clone)]
pub struct Error {
    pub code: i16,
    pub info: Option<String>,
    pub redirect: Option<String>,
}

impl Error {
    pub fn new(code: i16, info: &str, redirect: Option<String>) -> Self {
        Self {
            code,
            info: Some(info.to_string()),
            redirect,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}: {:?})", self.code, &self.info)
    }
}

impl std::error::Error for Error {}
impl warp::reject::Reject for Error {}

impl warp::Reply for Error {
    fn into_response(self) -> warp::reply::Response {
        if let Some(_url) = self.redirect {
            //return warp::redirect(warp::http::Uri::from_static(url.leak())).into_response();
        };
        let code = self.code;
        let info = self.info.unwrap_or(String::from("Unknown error"));

        let description = ERROR_CODE_INFO
            .iter()
            .find_map(|(code, info)| {
                if &self.code != code {
                    return None;
                }

                Some(*info)
            })
            .unwrap_or("Unknown error");

        warp::reply::html(format!(
            r#"
            <!DOCTYPE html>
            <html>
                <head>
                    <title>Error - {code}</title>
                    <link rel="stylesheet" href="static/index.css" />
                </head>
                <body>
                    <nav>

                    </nav>
                    <section class="content">
                        <h1>{info}</h1>
                        <p>{description}</p>
                    </section>
                </body>
            </html>
        "#
        ))
        .into_response()
    }
}

#[derive(Debug)]
pub struct TypeError {
    info: String,
}

impl TypeError {
    pub fn new(info: &str) -> Self {
        Self {
            info: info.to_string(),
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
