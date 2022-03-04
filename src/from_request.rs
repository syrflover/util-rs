use http::Request;
use hyper::Body;

#[async_trait::async_trait]
pub trait FromRequest<'a>: Sized {
    type Parameter: Send;
    type Error;

    async fn from_request(
        param: Self::Parameter,
        request: &'a mut Request<Body>,
    ) -> Result<Self, Self::Error>;
}

#[async_trait::async_trait]
pub trait ToPayload<'a, T> {
    type Parameter: Send;
    type Error;

    async fn to_payload(&'a mut self, param: Self::Parameter) -> Result<T, Self::Error>;
}

#[async_trait::async_trait]
impl<'a, T> ToPayload<'a, T> for Request<Body>
where
    T: FromRequest<'a> + 'static,
{
    type Parameter = T::Parameter;
    type Error = T::Error;

    async fn to_payload(&'a mut self, param: Self::Parameter) -> Result<T, Self::Error> {
        T::from_request(param, self).await
    }
}

#[async_trait::async_trait]
pub trait FromOwnedRequest: Sized {
    type Parameter: Send;
    type Error;

    async fn from_owned_request(
        param: Self::Parameter,
        request: Request<Body>,
    ) -> Result<Self, Self::Error>;
}

#[async_trait::async_trait]
pub trait IntoPayload<T> {
    type Parameter: Send;
    type Error;

    async fn into_payload(self, param: Self::Parameter) -> Result<T, Self::Error>;
}

#[async_trait::async_trait]
impl<T> IntoPayload<T> for Request<Body>
where
    T: FromOwnedRequest + 'static,
{
    type Parameter = T::Parameter;
    type Error = T::Error;

    async fn into_payload(self, param: Self::Parameter) -> Result<T, Self::Error> {
        T::from_owned_request(param, self).await
    }
}
