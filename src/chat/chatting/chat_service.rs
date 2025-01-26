use std::sync::Arc;

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

    pub async fn create_room(
        &self,
        app_state: ArcAppState,
        customer_id: i64,
    ) -> MangJooResult<ChatRoomId> {
        let create_room = ChatRoom::new(customer_id);

        let chat_room = async {
            let mut transaction = app_state.create_transaction().await?;
            let mut tx = transaction
                .begin()
                .await
                .map_err(|err| AppError::DatabaseError(err.to_string()))?;

            let chat_room = repository::save(create_room, &mut tx).await?;

            let _ = tx.commit().await;
            Ok(chat_room)
        }
        .await?;

        Ok(ChatRoomId::from(chat_room))
    }
}
