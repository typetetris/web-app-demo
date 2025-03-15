use actix_web::{App, HttpServer};
use tracing_actix_web::TracingLogger;

mod chat;
mod infrastructure;
mod services;
pub(crate) mod util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    infrastructure::setup_tracing_subscriber()?;

    HttpServer::new(|| {
        App::new()
            .wrap(TracingLogger::default())
            .service(services::hello)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;
    Ok(())
}
