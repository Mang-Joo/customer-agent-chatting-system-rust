use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
use chatting::chat_handler::{create_room, join_chat_room};

use crate::config::app_state::AppState;

pub mod agent;
pub mod chatting;
pub mod customer;

pub async fn create_chat_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/create/chat-room", post(create_room))
        .route("/join/chat-room/{room_id}", get(join_chat_room))
}
