use http::{header, Request};
use hyper::Body;
use serde::de::DeserializeOwned;

use crate::ReadChunks;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Not supported content-type: {0}")]
    NotSupportedContentType(String),

    #[error("Json deserialize: {0}")]
    JsonDeserialize(serde_json::Error),
}

#[async_trait::async_trait]
pub trait BodyParser<P>
where
    P: DeserializeOwned,
    Self: Sized,
{
    type Error;

    async fn body_parse(&mut self) -> Result<P, Self::Error>;
}

#[async_trait::async_trait]
impl<P> BodyParser<P> for Request<Body>
where
    P: DeserializeOwned,
    Self: Sized,
{
    type Error = Error;

    async fn body_parse(&mut self) -> Result<P, Self::Error> {
        let chunks = self.body_mut().read_chunks().await.unwrap_or_default();

        let content_type = self
            .headers()
            .get(header::CONTENT_TYPE)
            .map(|x| x.to_str().unwrap_or_default());

        match content_type {
            Some(content_type) if content_type.starts_with("application/json") => {
                let payload =
                    serde_json::from_slice::<P>(&chunks).map_err(Error::JsonDeserialize)?;

                Ok(payload)
            }
            content_type => {
                let content_type = content_type.unwrap_or_default().to_owned();
                Err(Error::NotSupportedContentType(content_type))
            }
        }
    }
}
