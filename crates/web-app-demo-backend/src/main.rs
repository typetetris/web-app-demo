use actix_web::{HttpServer, web};
use chat::ChatServer;

mod chat;
mod infrastructure;
mod services;
pub(crate) mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    infrastructure::setup_tracing_subscriber()?;

    let chat_server = ChatServer::new();
    let app_state = web::Data::new(chat_server);

    HttpServer::new(move || services::setup_app(app_state.clone()))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await?;
    Ok(())
}
