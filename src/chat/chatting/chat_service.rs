use std::sync::Arc;

use futures::FutureExt;
use sqlx::Acquire;

use crate::config::{app_state::ArcAppState, db::transaction, error::AppError, MangJooResult};

use super::{
    chat_room::ChatRoom,
    repository::{self, ChatRepository},
    ChatRoomId,
};

#[derive(Debug, Clone)]
pub struct ChatService {
    chat_repository: Arc<ChatRepository>,
}

impl ChatService {
    pub fn new(chat_repository: ChatRepository) -> Self {
        Self {
            chat_repository: Arc::new(chat_repository),
        }
    }

    pub async fn create_room(&self, customer_id: i64) -> MangJooResult<ChatRoomId> {
        let create_room = ChatRoom::new(customer_id);

        let chat_room = transaction(&self.chat_repository.pool, |tx| {
            repository::save(create_room, tx).boxed()
        })
        .await?;

        Ok(ChatRoomId::from(chat_room))
    }
}
