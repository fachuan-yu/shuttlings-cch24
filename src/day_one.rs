use actix_web::{get, HttpResponse, HttpResponseBuilder, http::StatusCode};

//-1
#[get("/")]
pub async fn hello_bird() -> &'static str {
    "Hello, bird!"
}

#[get("/-1/seek")]
pub async fn seek() -> HttpResponse {
    HttpResponseBuilder::new(StatusCode::from_u16(302).unwrap())
        .insert_header(("Location", "https://www.youtube.com/watch?v=9Gc4QTqslN4"))
        .finish()
}
