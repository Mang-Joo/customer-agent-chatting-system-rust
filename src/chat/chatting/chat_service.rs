use anyhow::{Ok, Result};

use super::{chat_room::ChatRooms, ChatRoomId};

pub async fn create_room(customer_id: i64, chat_rooms: &ChatRooms) -> Result<ChatRoomId> {
    let create_room = chat_rooms.create_room(customer_id).await?;

    Ok(ChatRoomId(create_room))
}
