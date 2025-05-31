use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

pub async fn handler(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    println!("✅ Running middleware");
    Ok(next.run(req).await)
}
