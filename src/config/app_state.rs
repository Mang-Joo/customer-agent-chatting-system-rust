use std::{collections::HashMap, sync::Arc};

use async_redis_session::RedisSessionStore;
use axum::extract::ws::{Message, WebSocket};
use sqlx::PgPool;
use tokio::sync::{broadcast, RwLock};

use crate::chat::{
    agent::agent::Agents,
    chatting::{chat_room::ChatRooms, ChatRoomId},
};

use super::session::SessionManager;

pub type ArcAppState = Arc<AppState>;

#[derive(Debug, Clone)]
pub struct AppState {
    // 활성화된 채팅방 관리
    pub rooms: ChatRooms,

    // 상담원 관리
    pub agents: Agents,

    // 대기열 관리 (상담원 배정 대기 중인 방들)
    pub waiting_queue: Arc<RwLock<Vec<ChatRoomId>>>,
    pub socket_rooms: Arc<RwLock<HashMap<ChatRoomId, broadcast::Sender<Message>>>>,
    pub db_pool: PgPool,
    pub session_store: SessionManager,
}

impl AppState {
    pub fn new(db_pool: PgPool, redis_session_store: RedisSessionStore) -> Self {
        Self {
            rooms: ChatRooms::new(),
            agents: Agents::new(),
            waiting_queue: Arc::new(RwLock::new(Vec::new())),
            socket_rooms: Arc::new(RwLock::new(HashMap::new())),
            db_pool,
            session_store: SessionManager::new(redis_session_store),
        }
    }
}

#[derive(Debug)]
pub struct RoomConnections {
    pub agent_sender: Option<WsSender>,
    pub customer_sender: Option<WsSender>,
}
type WsSender = futures::stream::SplitSink<WebSocket, Message>;
