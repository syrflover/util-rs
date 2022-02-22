use http::{header::HeaderName, HeaderValue, Response, StatusCode};

pub trait SetResponse<B> {
    type Error;

    fn set_status<T>(&mut self, code: T) -> Result<(), Self::Error>
    where
        StatusCode: TryFrom<T>,
        <StatusCode as TryFrom<T>>::Error: Into<Self::Error>;

    fn set_header<K, V>(&mut self, key: K, value: V) -> Result<(), Self::Error>
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<Self::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<Self::Error>;

    fn set_headers(&mut self, headers: impl Iterator<Item = (HeaderName, HeaderValue)>);

    fn set_body(&mut self, body: B);
}

impl<B> SetResponse<B> for Response<B> {
    type Error = http::Error;

    fn set_status<T>(&mut self, code: T) -> Result<(), Self::Error>
    where
        StatusCode: TryFrom<T>,
        <StatusCode as TryFrom<T>>::Error: Into<Self::Error>,
    {
        *self.status_mut() = code.try_into().map_err(Into::into)?;
        Ok(())
    }

    fn set_header<K, V>(&mut self, key: K, value: V) -> Result<(), Self::Error>
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<Self::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<Self::Error>,
    {
        let key: HeaderName = key.try_into().map_err(Into::into)?;
        let val = value.try_into().map_err(Into::into)?;

        self.headers_mut().append(key, val);

        Ok(())
    }

    fn set_headers(&mut self, headers: impl Iterator<Item = (HeaderName, HeaderValue)>) {
        let header_map = self.headers_mut();

        for (key, value) in headers {
            header_map.append(key, value);
        }
    }

    fn set_body(&mut self, body: B) {
        *self.body_mut() = body;
    }
}
