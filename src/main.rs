use axum_chatting_service::start_server;

#[tokio::main]
async fn main() {
    start_server().await;
}
