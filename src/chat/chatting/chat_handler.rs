use futures::{SinkExt, StreamExt};
use std::{ops::Not, sync::Arc};
use tokio::sync::broadcast;

use axum::{
    extract::{ws::Message, Path, State, WebSocketUpgrade},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::config::{app_state::AppState, error::AppError};

use super::{chat_service, ChatRoomId};

#[derive(Debug, Deserialize)]
pub struct CreateRoomRequest {
    customer_id: i64,
}

#[derive(Debug, Serialize)]
pub struct CreateRoomResponse {
    room_id: ChatRoomId,
}

#[tracing::instrument]
pub async fn create_room(
    State(app_state): State<Arc<AppState>>,
    Json(request): Json<CreateRoomRequest>,
) -> Result<Json<CreateRoomResponse>, AppError> {
    let result = chat_service::create_room(request.customer_id, &app_state.rooms).await;
    match result {
        Ok(room_id) => {
            let mut socket_room = app_state.socket_rooms.write().await;
            socket_room.insert(room_id.clone(), broadcast::channel(100).0);
            info!("Success Create Chat Room : {}", room_id.0);
            return Ok(Json(CreateRoomResponse { room_id: room_id }));
        }
        Err(error) => {
            tracing::error!("Can't make create room {:?}", error);
            return Err(AppError::InvalidRequest(error.to_string()));
        }
    }
}

#[tracing::instrument]
pub async fn join_chat_room(
    State(app_state): State<Arc<AppState>>,
    ws: WebSocketUpgrade,
    Path(room_id): Path<ChatRoomId>,
) -> impl IntoResponse {
    let is_available_room = app_state.rooms.is_available_room(room_id.clone()).await;
    if is_available_room.not() {
        return Err(AppError::RoomNotFound(format!(
            "The room does not exists. Room Id = {}",
            room_id.0
        )));
    }

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, Arc::clone(&app_state), room_id)))
}

async fn handle_socket(
    socket: axum::extract::ws::WebSocket,
    state: Arc<AppState>,
    room_id: ChatRoomId,
) {
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // 1. socket_room lock을 빨리 해제하기 위해 scope 사용
    let tx = {
        let socket_room = state.socket_rooms.read().await; // write 대신 read 사용
        socket_room.get(&room_id).expect("Room not found").clone()
    };
    let mut rx = tx.subscribe();

    let mut send_task = tokio::spawn(async move {
        println!("Starting send task"); // 디버그 로그
        while let Ok(message) = rx.recv().await {
            if ws_sender.send(message).await.is_err() {
                println!("Error sending message, closing send task"); // 에러 로그
                return;
            }
        }
    });

    let tx_clone = tx.clone();
    let mut receive_task = tokio::spawn(async move {
        println!("Starting receive task"); // 디버그 로그
        while let Some(Ok(message)) = ws_receiver.next().await {
            if let Message::Text(text) = message {
                if tx_clone.send(Message::Text(text)).is_err() {
                    println!("Error broadcasting message, closing receive task"); // 에러 로그
                    return;
                }
            }
        }
        println!("Client disconnected, closing receive task"); // 연결 종료 로그
    });

    tokio::select! {
        result = &mut send_task => {
            println!("Send task ended: {:?}", result);  // 종료 이유 로그
            receive_task.abort();
        },
        result = &mut receive_task => {
            println!("Receive task ended: {:?}", result);  // 종료 이유 로그
            send_task.abort();
        }
    };

    println!("WebSocket connection closed for room: {}", room_id.0);
    println!("Number of subscribers remaining: {}", tx.receiver_count());

    let _ = tx.send(Message::Text(
        format!("A user has disconnected from room {}", room_id.0).into(),
    ));
}
