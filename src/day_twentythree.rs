use actix_web::{cookie::Cookie, get, post, web, http::StatusCode,  HttpRequest, HttpResponse, Responder};
use log::{error, info};
use std::fs;
use std::io::Result;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use html_escape::encode_safe; // 引入 HTML 转义库



#[derive(Deserialize)]
struct PathParams {
    state: String,
    n: String,  // n is treated as a string
}


// Struct to store the ornament state
#[derive(Clone)]
pub struct OrnamentState {
    pub state: Arc<Mutex<String>>, // Mutex to allow safe mutable access across threads
}



#[get("/23/star")]
pub async fn light_star() -> impl Responder {

    // return html with <div class="lit" id="star"> </div>
    HttpResponse::Ok()
        .content_type("text/html")
        .body(r#"<div class="lit" id="star"> </div>"#)


}




#[get("/23/present/{color}")]
pub async fn present_color(path: web::Path<String>) -> impl Responder {
    let color = path.into_inner(); // 提取路径参数 {color}

    // 根据颜色选择不同的 HTML 响应
    let response_body = match color.as_str() {
        "red" => Some(r#"<div class="present red" hx-get="/23/present/blue" hx-swap="outerHTML">
                        <div class="ribbon"></div>
                        <div class="ribbon"></div>
                        <div class="ribbon"></div>
                        <div class="ribbon"></div>
                    </div>"#),
        "blue" => Some(r#"<div class="present blue" hx-get="/23/present/purple" hx-swap="outerHTML">
                        <div class="ribbon"></div>
                        <div class="ribbon"></div>
                        <div class="ribbon"></div>
                        <div class="ribbon"></div>
                    </div>"#),
        "purple" => Some(r#"<div class="present purple" hx-get="/23/present/red" hx-swap="outerHTML">
                        <div class="ribbon"></div>
                        <div class="ribbon"></div>
                        <div class="ribbon"></div>
                        <div class="ribbon"></div>
                    </div>"#),
        _ => None,
    };

    match response_body {
        Some(body) => HttpResponse::Ok()
        .content_type("text/html")
        .body(body),
        None => HttpResponse::ImATeapot()
        .content_type("text/plain")
        .body("Invalid color: I'm a teaport!"),

    }

}



#[get("/23/ornament/{state}/{n}")]
pub async fn ornament_color(path: web::Path<PathParams>, data: web::Data<OrnamentState>) -> impl Responder {
    let state = &path.state; // 提取路径参数 {state}
    let n = &path.n; // 提取路径参数 {number}
    info!("state is {}, n is {}", state, n);

    let escaped_state = encode_safe(state).to_string();
    let escaped_n = encode_safe(n).to_string();



    // Get the current state
    let mut ornament_state_lock = data.state.lock().await;



    // Check if the state is valid
    if state != "on" && state != "off" {
        return HttpResponse::build(StatusCode::IM_A_TEAPOT)
            .content_type("text/plain")
            .body("418 I'm a teapot: Invalid state.");
    }

    
    // Determine the CSS class based on the state
    let class = if state == "on" { " on" } else { "" };

    // Switch state for the next request (if it's on, set it to off, and vice versa)
    let new_state = if state == "on" { "off" } else { "on" };
    *ornament_state_lock = new_state.to_string();

    

    // Generate the HTML for the ornament with the given n and state
    let html = format!(
        r#"<div class="ornament{class}" id="ornament{escaped_n}" hx-trigger="load delay:2s once" hx-get="/23/ornament/{}/{escaped_n}" hx-swap="outerHTML"></div>"#,
        encode_safe(&new_state).to_string()
    );

    // Return the generated HTML
    HttpResponse::Ok().content_type("text/html").body(html)

}



