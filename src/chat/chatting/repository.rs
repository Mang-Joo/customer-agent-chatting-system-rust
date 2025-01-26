use std::{future::Future, str::FromStr};

use chrono::NaiveDateTime;
use sqlx::{query_as, PgPool, PgTransaction, Postgres, Transaction};
use uuid::Uuid;

use crate::config::{error::AppError, MangJooResult};

use super::chat_room::{ChatRoom, RoomStatus};

#[derive(Debug, Clone)]
pub struct ChatRepository {
    pub pool: PgPool,
}

impl ChatRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save(
        &self,
        chat_room: ChatRoom,
        transaction: &mut PgTransaction<'_>,
    ) -> MangJooResult<ChatRoom> {
        let chat_room_entity = query_as!(
            ChatRoomEntity,
            "
            INSERT INTO CHAT_ROOM (room_id, customer_id, agent_id, status)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            ",
            chat_room.room_id(),
            chat_room.customer_id(),
            chat_room.agent_id(),
            chat_room.status().to_string()
        )
        .fetch_one(transaction.as_mut())
        .await
        .map_err(|err| AppError::DatabaseError(format!("DB Error {:?}", err)))?;

        Ok(chat_room_entity.into())
    }
}

pub struct ChatRoomEntity {
    id: i64,
    room_id: Uuid,
    customer_id: i64,
    agent_id: Option<i64>,
    status: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

impl From<ChatRoom> for ChatRoomEntity {
    fn from(value: ChatRoom) -> Self {
        Self {
            id: value.id(),
            room_id: value.room_id().to_owned(),
            customer_id: value.customer_id(),
            agent_id: value.agent_id(),
            status: value.status().to_string(),
            created_at: value.created_at().naive_utc(),
            updated_at: value.updated_at().naive_utc(),
        }
    }
}

impl Into<ChatRoom> for ChatRoomEntity {
    fn into(self) -> ChatRoom {
        ChatRoom::from(
            self.id,
            self.room_id,
            self.customer_id,
            self.agent_id,
            RoomStatus::from_str(self.status.as_str()).unwrap(),
            self.created_at.and_utc(),
            self.updated_at.and_utc(),
        )
    }
}

pub async fn save(
    chat_room: ChatRoom,
    transaction: &mut PgTransaction<'_>,
) -> MangJooResult<ChatRoom> {
    let chat_room_entity = query_as!(
        ChatRoomEntity,
        "
        INSERT INTO CHAT_ROOM (room_id, customer_id, agent_id, status)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        ",
        chat_room.room_id(),
        chat_room.customer_id(),
        chat_room.agent_id(),
        chat_room.status().to_string()
    )
    .fetch_one(transaction.as_mut())
    .await
    .map_err(|err| AppError::DatabaseError(format!("DB Error {:?}", err)))?;

    Ok(chat_room_entity.into())
}

#[cfg(test)]
mod chat_repository_test {
    use dotenv::dotenv;
    use sqlx::PgPool;

    use crate::chat::chatting::chat_room::ChatRoom;

    use super::ChatRepository;

    #[sqlx::test]
    async fn insert_chant_room_test() {
        dotenv().ok();
        let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
            .await
            .unwrap();

        let mut tx = pool.begin().await.unwrap();
        let repository = ChatRepository::new(pool.clone());

        let chat_room = ChatRoom::new(1);
        let result = repository.save(chat_room, &mut tx).await;

        assert!(result.is_ok());
        let saved_room = result.unwrap();
        assert_eq!(saved_room.customer_id(), 1);

        tx.rollback().await.unwrap();
    }
}
