mod create;

use crate::middlewares::{with_auth, with_is_admin};
use crate::AppState;

use axum::{middleware, routing::post, Router};
use std::sync::Arc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/create", post(create::handler))
        .layer(middleware::from_fn(with_is_admin::handler))
        .layer(middleware::from_fn(with_auth::handler))
}
