use axum::body::Body;
use axum::http::Request;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response as AxumResponse};
use bytes::Bytes;
use futures_util::stream;
use futures_util::StreamExt;

pub async fn inject_doctype(
    req: Request<Body>,
    next: Next,
) -> Result<AxumResponse, axum::http::StatusCode> {
    let res = next.run(req).await;

    if let Some(content_type) = res.headers().get(axum::http::header::CONTENT_TYPE) {
        if let Ok(ct_str) = content_type.to_str() {
            if ct_str.contains("text/html") {
                let (parts, body) = res.into_parts();
                let prefix = stream::once(async {
                    Ok::<Bytes, axum::Error>(Bytes::from_static(b"<!DOCTYPE html>"))
                });
                let new_body = Body::from_stream(prefix.chain(body.into_data_stream()));
                let new_res = axum::http::Response::from_parts(parts, new_body);
                return Ok(new_res.into_response());
            }
        }
    }

    Ok(res.into_response())
}
