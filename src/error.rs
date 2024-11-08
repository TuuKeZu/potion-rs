use std::{fmt::{self, Debug, Display}, marker::PhantomData};

use http::{Response, StatusCode};
use warp::{reject::Rejection};
pub trait IntoErrorPage {
    fn into_html(error: &Error) -> String;
}

pub trait ErrorPage: IntoErrorPage + Debug + Sync + Send {}

#[derive(Debug)]
struct DefaultErrorPage {}
impl IntoErrorPage for DefaultErrorPage {
    fn into_html(error: &Error) -> String {
        format!("Error {}: {:#?}", error.code, error.info)
    }
}

impl ErrorPage for DefaultErrorPage {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HtmlError {
    Unauthorized,
    InvalidRequest,
    InvalidSession,
    InternalServerError,
}

impl HtmlError {
    pub fn default(self) -> Error {
        match self {
            HtmlError::InvalidSession => Error::new(401, "Invalid credentials", None),
            HtmlError::Unauthorized => Error::new(403, "Permission denied", None),
            HtmlError::InvalidRequest => Error::new(400, "Invalid request", None),
            HtmlError::InternalServerError => Error::new(500, "Internal server error", None),
        }
    }

    pub fn new(self, info: &str) -> Error {
        match self {
            Self::InvalidSession => Error::new(401, info, None),
            Self::Unauthorized => Error::new(403, info, None),
            Self::InvalidRequest => Error::new(400, info, None),
            Self::InternalServerError => Error::new(500, info, None),
        }
    }

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


#[derive(Debug)]
pub struct CustomError<P> {
    _inner: Error,
    _kind: PhantomData<P>
}

impl<P: ErrorPage> From<Error> for CustomError<P> {
    fn from(value: Error) -> Self {
        Self {
            _inner: value,
            _kind: PhantomData
        }
    }
}

impl<P: ErrorPage> Display for CustomError<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}: {:?})", self._inner.code, &self._inner.info)
    }
}

impl<P: ErrorPage> std::error::Error for CustomError<P> {}
impl<P: ErrorPage + 'static> warp::reject::Reject for CustomError<P> {}


impl<P: ErrorPage> warp::Reply for CustomError<P> {
    fn into_response(self) -> warp::reply::Response {
        if let Some(url) = self._inner.redirect {
            return warp::reply::with_header(
                warp::redirect(warp::http::Uri::from_static(url.leak())),
                "Cache-Control",
                "no-cache, must-revalidate",
            )
            .into_response();
        };

        warp::reply::html(P::into_html(&self._inner)).into_response()
    }
}

impl<P: ErrorPage> Into<http::StatusCode> for CustomError<P> {
    fn into(self) -> http::StatusCode {
        StatusCode::from_u16(self._inner.code as u16).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

#[derive(Debug)]
pub struct Error {
    pub code: i16,
    pub info: Option<String>,
    pub redirect: Option<String>
}

impl Error {
    pub fn new(code: i16, info: &str, redirect: Option<String>) -> Self {
        Self {
            code,
            info: Some(info.to_string()),
            redirect
        }
    }

    pub fn custom(code: i16, info: &str, redirect: Option<String>) -> Self {
        Self {
            code,
            info: Some(info.to_string()),
            redirect
        }
    }

    pub fn with_page(self) -> Self {
        Self {
            code: self.code,
            info: self.info,
            redirect: self.redirect
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
        if let Some(url) = self.redirect {
            return warp::reply::with_header(
                warp::redirect(warp::http::Uri::from_static(url.leak())),
                "Cache-Control",
                "no-cache, must-revalidate",
            )
            .into_response();
        };

        warp::reply::html(DefaultErrorPage::into_html(&self)).into_response()
    }
}

impl Into<http::StatusCode> for Error {
    fn into(self) -> http::StatusCode {
        StatusCode::from_u16(self.code as u16).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
}