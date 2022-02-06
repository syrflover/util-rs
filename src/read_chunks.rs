#[async_trait::async_trait]
pub trait ReadChunks {
    type Error;

    async fn read_chunks(&mut self) -> Result<Vec<u8>, Self::Error>;
}

#[async_trait::async_trait]
impl ReadChunks for hyper::Body {
    type Error = hyper::Error;

    async fn read_chunks(&mut self) -> Result<Vec<u8>, Self::Error> {
        let mut chunks = Vec::new();

        while let Some(chunk) = hyper::body::HttpBody::data(self).await {
            let chunk = chunk?;

            chunks.push(chunk);
        }

        Ok(chunks.concat())
    }
}
