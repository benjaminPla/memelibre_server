mod create;

use crate::AppState;

use axum::{routing::post, Router};
use std::sync::Arc;



pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/create", post(create::handler))
}
