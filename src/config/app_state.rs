use std::{collections::HashMap, sync::Arc};

use async_redis_session::RedisSessionStore;
use axum::extract::ws::{Message, WebSocket};
use sqlx::{pool::PoolConnection, PgPool, PgTransaction, Postgres};
use tokio::sync::{broadcast, RwLock};

use crate::chat::chatting::ChatRoomId;

use super::{error::AppError, session::SessionManager, MangJooResult};

pub type ArcAppState = Arc<AppState>;

#[derive(Debug, Clone)]
pub struct AppState {
    pub waiting_queue: Arc<RwLock<Vec<ChatRoomId>>>,
    pub socket_rooms: Arc<RwLock<HashMap<ChatRoomId, broadcast::Sender<Message>>>>,
    pub db_pool: PgPool,
    pub session_store: SessionManager,
}

impl AppState {
    pub fn new(db_pool: PgPool, redis_session_store: RedisSessionStore) -> Self {
        Self {
            waiting_queue: Arc::new(RwLock::new(Vec::new())),
            socket_rooms: Arc::new(RwLock::new(HashMap::new())),
            db_pool,
            session_store: SessionManager::new(redis_session_store),
        }
    }

    pub async fn create_transaction(&self) -> MangJooResult<PoolConnection<Postgres>> {
        let pool_connection = self.db_pool.acquire().await;
        let pool_connection = pool_connection
            .map_err(|err| AppError::DatabaseError(format!("DB Connection Error {}", err)))?;

        Ok(pool_connection)
    }
}

#[derive(Debug)]
pub struct RoomConnections {
    pub agent_sender: Option<WsSender>,
    pub customer_sender: Option<WsSender>,
}
type WsSender = futures::stream::SplitSink<WebSocket, Message>;
