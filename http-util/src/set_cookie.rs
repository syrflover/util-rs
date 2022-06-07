use std::collections::HashMap;

use http::{
    header::{self, HeaderName},
    HeaderMap, HeaderValue,
};

#[cfg_attr(test, derive(PartialEq))]
#[derive(Default, Clone, Debug)]
pub struct SetCookieOptions {
    pub http_only: bool,
    pub secure: bool,
    // expires: ,
    /// Seconds
    pub max_age: Option<i64>,
    pub domain: Option<String>,
    pub path: Option<String>,
}

impl SetCookieOptions {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn is_set_cookie_option(st: &str) -> bool {
        let st = st.to_lowercase();

        st.starts_with("max-age=")
            || st.starts_with("domain=")
            || st.starts_with("path=")
            // || st.starts_with("expires=")
            || st.eq("httponly")
            || st.eq("secure")
    }

    pub fn http_only(mut self, http_only: bool) -> Self {
        self.http_only = http_only;

        self
    }

    #[allow(dead_code)]
    pub fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;

        self
    }

    pub fn max_age(mut self, max_age: i64) -> Self {
        self.max_age.replace(max_age);

        self
    }

    pub fn domain(mut self, domain: impl Into<String>) -> Self {
        self.domain.replace(domain.into());

        self
    }

    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path.replace(path.into());

        self
    }
}

impl<'a> From<Vec<&'a str>> for SetCookieOptions {
    fn from(xs: Vec<&'a str>) -> Self {
        let mut options = SetCookieOptions {
            domain: None,
            max_age: None,
            path: None,
            http_only: false,
            secure: false,
        };

        for st in xs.iter().map(|st| st.to_lowercase()) {
            if st.starts_with("domain=") {
                let domain = st.split('=').nth(1).unwrap_or_default();
                options.domain.replace(domain.to_string());
            } else if st.starts_with("max-age=") {
                let max_age = st
                    .split('=')
                    .nth(1)
                    .and_then(|x| x.parse().ok())
                    .unwrap_or_default();
                options.max_age.replace(max_age);
            } else if st.starts_with("path=") {
                let path = st.split('=').nth(1).unwrap_or("/");
                options.path.replace(path.to_string());
            } else if st.starts_with("httponly") {
                options.http_only = true;
            } else if st.starts_with("secure") {
                options.secure = true;
            }
        }

        options
    }
}

#[cfg_attr(test, derive(PartialEq))]
#[derive(Default, Debug)]
pub struct SetCookie {
    inner: HashMap<String, (String, SetCookieOptions)>,
}

impl SetCookie {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn from_headers(headers: &HeaderMap) -> Self {
        headers
            .iter()
            .filter(|(k, _)| k == &header::SET_COOKIE)
            .filter_map(|(_, v)| v.to_str().ok())
            .into()
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.inner.get(key).map(|(v, _)| v.as_str())
    }

    pub fn take(&mut self, key: &str) -> Option<String> {
        self.inner.remove(key).map(|(x, _)| x)
    }

    pub fn set(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
        options: SetCookieOptions,
    ) -> Self {
        self.inner.insert(key.into(), (value.into(), options));

        self
    }

    #[allow(dead_code)]
    pub fn remove(mut self, key: impl Into<String>) -> Self {
        self.inner.remove(&key.into());

        self
    }

    /// SetHeaders::headers(set_cookie.iter());
    pub fn iter(&self) -> impl Iterator<Item = (HeaderName, HeaderValue)> + '_ {
        self.inner
            .iter()
            .map(|(key, (value, options))| fmt(key, value, options))
            .map(|st| (header::SET_COOKIE, st.parse().unwrap()))
    }
}

fn fmt(
    key: &str,
    value: &str,
    SetCookieOptions {
        domain,
        max_age,
        http_only,
        secure,
        path,
    }: &SetCookieOptions,
) -> String {
    let mut base = format!("{}={}", key, value);

    if let Some(domain) = domain {
        base = format!("{}; Domain={}", base, domain);
    }

    if let Some(max_age) = max_age {
        base = format!("{}; Max-Age={}", base, max_age);
    }

    if let Some(path) = path {
        base = format!("{}; Path={}", base, path);
    }

    if *http_only {
        base = format!("{}; HttpOnly", base);
    }

    if *secure {
        base = format!("{}; Secure", base);
    }

    base
}

impl<A, I> From<I> for SetCookie
where
    A: AsRef<str>,
    I: Iterator<Item = A>,
{
    fn from(it: I) -> Self {
        // Set-Cookie: key=value; Max-Age=12345; Domain=eeeee.com; HttpOnly; Secure

        let mut set_cookie = Self::new();

        for header_value in it {
            let (options, key_value): (Vec<_>, Vec<_>) = header_value
                .as_ref()
                .split(';')
                .map(|st| st.trim())
                .partition(|st| SetCookieOptions::is_set_cookie_option(st));

            // println!("options = {:?}", options);
            // println!("key_value = {:?}", key_value);

            let mut key_value = key_value.get(0).map(|st| st.split('='));

            let key = key_value.as_mut().and_then(|st| st.next());
            let value = key_value.as_mut().and_then(|st| st.next());

            let (key, value) = match (key, value) {
                (Some(key), Some(value)) => (key, value),
                _ => continue,
            };

            // println!("key = {}", key);
            // println!("value = {}", value);

            set_cookie
                .inner
                .insert(key.to_string(), (value.to_string(), options.into()));
        }

        set_cookie
    }
}

#[test]
fn set_cookie_from_header_values() {
    let header_value = "key=value; Max-Age=12345; Domain=eeee.com; HttpOnly; Secure; Path=/abcd/e";

    let it = vec![header_value];

    let set_cookie = SetCookie::from(it.iter());

    // println!("{:?}", set_cookie);

    let expected = SetCookie::new().set(
        "key",
        "value",
        SetCookieOptions::new()
            .http_only(true)
            .secure(true)
            .max_age(12345)
            .domain("eeee.com")
            .path("/abcd/e"),
    );

    assert_eq!(set_cookie, expected);
}
