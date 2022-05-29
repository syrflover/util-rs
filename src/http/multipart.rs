use http::{
    header::{self, HeaderName},
    HeaderMap, HeaderValue, Request,
};
use hyper::Body;

use crate::ReadChunks;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Boundary")]
    Boundary,
    #[error("Body: {0}")]
    Body(#[from] hyper::Error),
    #[error("Not multipart")]
    NotMultipart,
}

pub struct Multipart {
    boundary: Vec<u8>,
    buf: Vec<u8>,
    pos: usize,
    end: usize,
}

impl Multipart {
    pub async fn new(request: &mut Request<Body>) -> Result<Multipart, Error> {
        let content_type = request
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|x| x.to_str().ok())
            .unwrap_or_default();

        if !content_type.contains("multipart/form-data") {
            return Err(Error::NotMultipart);
        }

        let boundary = match content_type.splitn(2, "boundary=").last() {
            Some(x) => [&[45, 45], x.as_bytes()]
                .into_iter()
                .flatten()
                .copied()
                .collect::<Vec<u8>>(),
            None => return Err(Error::Boundary),
        };

        let buf = request.body_mut().read_chunks().await?;

        let pos = twoway::find_bytes(&buf, &boundary).unwrap() + boundary.len(); // ignore first boundary
        let end = twoway::rfind_bytes(&buf, &boundary).unwrap(); // end boundary position

        // println!("pos = {pos}");
        // println!("end = {end}");

        Ok(Self {
            boundary,
            buf,
            pos,
            end,
        })
    }
}

impl Iterator for Multipart {
    type Item = (HeaderMap, Vec<u8>);

    fn next(&mut self) -> Option<Self::Item> {
        let end = self.pos + twoway::find_bytes(&self.buf[self.pos..], &self.boundary)?;

        // println!("start = {}", self.pos);
        // println!("end = {end}");

        let r = &self.buf[self.pos..end];

        // println!("{}", String::from_utf8(r.to_vec()).unwrap());

        // println!("{r:?}");

        if r.is_empty() || self.pos >= self.end {
            // println!("start = {}", self.pos);
            // println!("end = {end}");

            return None;
        }

        self.pos = end + self.boundary.len();

        let mut headers = HeaderMap::new();

        let crlf_pos = twoway::find_bytes(r, b"\r\n\r\n").unwrap();

        if let Ok(x) = String::from_utf8(r[..crlf_pos].to_vec()) {
            let lines = x.trim().lines();
            for x in lines {
                let mut it = x.splitn(2, ':');

                let header = it
                    .next()
                    .and_then(|name| {
                        it.next()
                            .map(|value| (name.to_owned(), value.trim().to_owned()))
                    })
                    .and_then(|(name, value)| {
                        Some((
                            HeaderName::from_bytes(name.as_bytes()).ok()?,
                            HeaderValue::from_str(&value).ok()?,
                        ))
                    });

                if let Some((name, value)) = header {
                    headers.insert(name, value);
                }
            }
        }

        // boundary_pos + length of "\r\n\r\n"
        let body = r[(crlf_pos + 4)..].to_vec();

        // println!("headers = {}", String::from_utf8(headers.to_vec()).unwrap());
        // println!("body = {}", String::from_utf8(body.to_vec()).unwrap());

        Some((headers, body))
    }
}

#[tokio::test]
async fn test_multipart() {
    let boundary = "abhjdahkdhfsikldhjfliawefrkhkahskda";

    let body = format!(
        r#"--{boundary}
Content-Type: application/http
Content-ID: response-

HTTP/1.1 200 OK
Content-Type: application/json; charset=UTF-8
Vary: Origin
Vary: X-Origin
Vary: Referer

{{
"name": "projects/35006771263/messages/0:1570471792141125%43c11b7043c11b70"
}}

--{boundary}
Content-Type: application/http
Content-ID: response-

HTTP/1.1 400 BAD REQUEST
Content-Type: application/json; charset=UTF-8
Vary: Origin
Vary: X-Origin
Vary: Referer

{{
"error": {{
    "code": 400,
    "message": "The registration token is not a valid FCM registration token",
    "status": "INVALID_ARGUMENT"
  }}
}}

--{boundary}
Content-Type: application/http
Content-ID: response-

HTTP/1.1 200 OK
Content-Type: application/json; charset=UTF-8
Vary: Origin
Vary: X-Origin
Vary: Referer

{{adnsdkjasdh
"name": "projects/35006771263/messages/0:1570471792141696%43c11b7043c11b70"
}}

--{boundary}--"#
    )
    .replace('\n', "\r\n")
    .replace("adnsdkjasdh", "\r\r\r\r\r\r\r\r");

    let mut request = Request::builder()
        .header(
            header::CONTENT_TYPE,
            format!("multipart/form-data; boundary={boundary}"),
        )
        .body(body.into())
        .unwrap();

    let multipart = Multipart::new(&mut request).await.unwrap();

    // let a = String::from_utf8(multipart.next().unwrap()).unwrap();

    // println!("{a}");

    for (headers, a) in multipart {
        let content_type = headers.get("Content-Type").unwrap().to_str().unwrap();
        let content_id = headers.get("Content-ID").unwrap().to_str().unwrap();

        println!("Content-Type: {content_type}");
        println!("Content-ID: {content_id}");
        println!();

        let a = String::from_utf8(a).unwrap();

        let r = a.contains('\r');

        assert!(r);

        println!("{a}");

        // println!("{}", a.replace('\r', "\\r"));

        println!("------------------");
    }
}
