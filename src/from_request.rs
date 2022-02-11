use http::Request;
use hyper::Body;

#[async_trait::async_trait]
pub trait FromRequest: Sized {
    type Parameter: Send;
    type Error;

    async fn from_request(
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
impl<'a, T> IntoPayload<T> for Request<Body>
where
    T: FromRequest + 'static,
{
    type Parameter = T::Parameter;
    type Error = T::Error;

    async fn into_payload(self, param: Self::Parameter) -> Result<T, Self::Error> {
        T::from_request(param, self).await
    }
}
