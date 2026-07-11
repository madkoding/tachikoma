use axum::{body::Body, extract::Request, middleware::Next, response::Response};

pub async fn request_id_middleware(mut request: Request<Body>, next: Next) -> Response {
    let request_id = uuid::Uuid::new_v4().to_string();

    request.headers_mut().insert(
        "x-request-id",
        request_id.parse().unwrap(),
    );

    let mut response = next.run(request).await;

    response.headers_mut().insert(
        "x-request-id",
        request_id.parse().unwrap(),
    );

    response
}