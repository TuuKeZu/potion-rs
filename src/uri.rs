use std::collections::HashMap;

use uri::Uri;
use warp::http::uri;

pub struct Builder {
    query: HashMap<String, String>,
    location: String,
}

impl Builder {
    pub fn new(location: &str) -> Self {
        Self {
            location: String::from(location),
            query: HashMap::new(),
        }
    }

    pub fn query_add(mut self, key: &str, value: &str) -> Self {
        self.query
            .insert(String::from(key), urlencoding::encode(value).into_owned());
        self
    }

    pub fn build(self) -> Uri {
        let uri = self
            .query
            .iter()
            .enumerate()
            .fold(self.location, |uri, (i, (k, v))| {
                if i == 0 {
                    return uri + &format!("?{k}={v}");
                }

                uri + &format!("&{k}={v}")
            });

        uri::Builder::new()
            .path_and_query(uri)
            .build()
            .expect("Invalid URI")
    }
}
