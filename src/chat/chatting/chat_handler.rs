use futures::{SinkExt, StreamExt, TryFutureExt};
use std::{ops::Not, sync::Arc};
use tokio::sync::broadcast;

use axum::{
    extract::{ws::Message, Path, State, WebSocketUpgrade},
    response::IntoResponse,
    Extension, Json,
};
use serde::Serialize;
use tracing::info;

use crate::config::{
    app_state::ArcAppState,
    error::AppError,
    session::{AuthUser, RequiredUser, UserSession},
    MangJooResult,
};

use super::{chat_service::ChatService, ChatRoomId};

#[derive(Debug, Serialize)]
pub struct CreateRoomResponse {
    room_id: ChatRoomId,
}

#[tracing::instrument]
pub async fn create_room(
    State(app_state): State<ArcAppState>,
    Extension(service): Extension<ChatService>,
    RequiredUser(session): RequiredUser,
) -> Result<Json<CreateRoomResponse>, AppError> {
    let result = service
        .create_room(Arc::clone(&app_state), session.user_id)
        .await;

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
    State(app_state): State<ArcAppState>,
    ws: WebSocketUpgrade,
    Path(room_id): Path<ChatRoomId>,
    AuthUser(user_session): AuthUser,
) -> impl IntoResponse {
    // let is_available_room = app_state
    //     .rooms
    //     .is_available_room(&room_id, user_session.user_id)
    //     .await;
    // if is_available_room.not() {
    //     return Err(AppError::RoomNotFound(format!(
    //         "The room does not exists. Room Id = {}",
    //         room_id.0
    //     )));
    // }

    // Ok(ws.on_upgrade(move |socket| {
    //     handle_socket(socket, Arc::clone(&app_state), room_id, user_session)
    //         .unwrap_or_else(|err| eprintln!("{}", err))
    // }))
}

async fn handle_socket(
    socket: axum::extract::ws::WebSocket,
    state: ArcAppState,
    room_id: ChatRoomId,
    user_session: UserSession,
) -> MangJooResult<()> {
    let (mut ws_sender, mut ws_receiver) = socket.split();

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
                if text.to_string() == "종료".to_string() {
                    let _ = tx_clone
                        .send(Message::Text(text))
                        .map_err(|err| AppError::InternalError(err.to_string()));
                    println!("Chat End");
                    return;
                }

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

    if tx.receiver_count() <= 1 {
        let _ = state.socket_rooms.write().await.remove(&room_id);
        // let _ = state.rooms.remove_room(&room_id);
    };

    let _ = tx.send(Message::Text(
        format!("A user has disconnected from room {}", room_id.0).into(),
    ));
    Ok(())
}
