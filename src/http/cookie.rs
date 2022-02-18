use std::{collections::HashMap, fmt::Display};

use http::{
    header::{self, HeaderName, HeaderValue},
    HeaderMap, Request,
};
use itertools::Itertools;

#[derive(Debug, Default)]
pub struct Cookie {
    inner: HashMap<String, String>,
}

impl Cookie {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn parse(cookie_str: &str) -> Self {
        let mut inner = HashMap::new();

        // madome_access_token=avchdef; madome_refresh_token=qwehkdfsjd
        for a in cookie_str.split(';') {
            let mut a = a.split('=');

            if let Some(key) = a.next() {
                let value = a.next().unwrap_or("");

                inner.insert(key.trim().to_string(), value.to_string());
            }
        }

        Self { inner }
    }

    pub fn add(&mut self, key: &str, value: &str) {
        self.inner.insert(key.to_string(), value.to_string());
    }

    #[allow(dead_code)]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.inner.get(key).map(|st| st as &str)
    }

    pub fn take(&mut self, key: &str) -> Option<String> {
        self.inner.remove(key)
    }
}

impl<T> From<&Request<T>> for Cookie {
    fn from(request: &Request<T>) -> Self {
        request.headers().into()
    }
}

impl From<&HeaderMap> for Cookie {
    fn from(headers: &HeaderMap) -> Self {
        let cookie_str = headers
            .get(header::COOKIE)
            .and_then(|a| a.to_str().ok())
            .unwrap_or("");

        Self::parse(cookie_str)
    }
}

impl Display for Cookie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let r = self
            .inner
            .iter()
            .map(|(key, value)| format!("{}={}", key, value))
            .join(";");

        write!(f, "{}", r)
    }
}

impl FromIterator<(String, String)> for Cookie {
    fn from_iter<T: IntoIterator<Item = (String, String)>>(iter: T) -> Self {
        Self {
            inner: HashMap::from_iter(iter),
        }
    }
}

impl<'a> FromIterator<(&'a str, &'a str)> for Cookie {
    fn from_iter<T: IntoIterator<Item = (&'a str, &'a str)>>(iter: T) -> Self {
        Self::from_iter(
            iter.into_iter()
                .map(|(key, value)| (key.to_string(), value.to_string())),
        )
    }
}

impl<'a> FromIterator<(&'a str, String)> for Cookie {
    fn from_iter<T: IntoIterator<Item = (&'a str, String)>>(iter: T) -> Self {
        Self::from_iter(
            iter.into_iter()
                .map(|(key, value)| (key.to_string(), value)),
        )
    }
}

impl<'a> FromIterator<(String, &'a str)> for Cookie {
    fn from_iter<T: IntoIterator<Item = (String, &'a str)>>(iter: T) -> Self {
        Self::from_iter(
            iter.into_iter()
                .map(|(key, value)| (key, value.to_string())),
        )
    }
}

impl From<Cookie> for (HeaderName, HeaderValue) {
    fn from(cookie: Cookie) -> Self {
        (
            header::COOKIE,
            HeaderValue::from_str(&cookie.to_string()).unwrap(),
        )
    }
}
