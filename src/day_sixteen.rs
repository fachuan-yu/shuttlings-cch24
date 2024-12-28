

use actix_web::{post, get, web, HttpRequest, HttpResponse, Responder};



//actix_web response

#[post("/16/wrap")]
pub async fn wrap_gift(
    body: String,
    req: HttpRequest,
) -> impl Responder {




    return HttpResponse::Ok().body("hello");


}


#[get("/16/unwrap")]
pub async fn unwrap_gift(
    body: String,
    req: HttpRequest,
) -> impl Responder {




    return HttpResponse::Ok().body("hello");


}


