use actix_web::{HttpResponse, Responder, get};
use tracing::instrument;

#[get("/")]
#[instrument(name = "hello handler")]
pub async fn hello() -> impl Responder {
    tracing::info!("hellow world");
    HttpResponse::Ok().body("Hello World!")
}
