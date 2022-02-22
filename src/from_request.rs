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
/*
#[async_trait::async_trait]
pub trait FromRefRequest: Sized {
    type Parameter: Send;
    type Error;

    async fn from_ref_request(
        param: Self::Parameter,
        request: &mut Request<Body>,
    ) -> Result<Self, Self::Error>;
}

#[async_trait::async_trait]
pub trait ToPayload<T> {
    type Parameter: Send;
    type Error;

    async fn to_payload(&mut self, param: Self::Parameter) -> Result<T, Self::Error>;
}

#[async_trait::async_trait]
impl<T> ToPayload<T> for Request<Body>
where
    T: FromRefRequest + 'static,
{
    type Parameter = T::Parameter;
    type Error = T::Error;

    async fn to_payload(&mut self, param: Self::Parameter) -> Result<T, Self::Error> {
        T::from_ref_request(param, self).await
    }
}
 */
