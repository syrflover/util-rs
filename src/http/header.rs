use http::{header::HeaderName, response::Builder as ResponseBuilder, HeaderValue};

pub trait SetHeaders {
    fn headers(self, headers: impl Iterator<Item = (HeaderName, HeaderValue)>) -> Self;
}

impl SetHeaders for ResponseBuilder {
    fn headers(self, headers: impl Iterator<Item = (HeaderName, HeaderValue)>) -> Self {
        let a = headers.fold(self, |res, (key, value)| res.header(key, value));

        println!("{:?}", a);

        a
    }
}
