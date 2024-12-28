use actix_web::{web, middleware::Logger,  web::ServiceConfig, App, };


use shuttle_actix_web::ShuttleActixWeb;


mod day_one;
pub use crate::day_one::{hello_bird, seek};

mod day_two;
pub use crate::day_two::{dest, key, v6_dest, v6_key};


mod day_five;
pub use crate::day_five::handle_manifest;

mod day_nine;
pub use crate::day_nine::handle_milk_request;
use leaky_bucket::RateLimiter;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Duration;
pub use crate::day_nine::handle_milk_refill;


mod day_twelve;
pub use crate::day_twelve::handle_board;
pub use crate::day_twelve::board_reset;
pub use crate::day_twelve::place_cookie_milk;
pub use crate::day_twelve::random_board;
use day_twelve:: Board;


mod day_sixteen;
pub use crate::day_sixteen::wrap_gift;
pub use crate::day_sixteen::unwrap_gift;




#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    //env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    env_logger::Env::new().default_filter_or("info");

    let _app = App::new()
        //.wrap(Logger::new("%a %{User-Agent}i"));
        .wrap(Logger::default());

    // day9
    let bucket = RateLimiter::builder()
        .max(5) // 最大令牌数为 5
        .initial(5) // 初始化时有 5 个令牌
        .interval(Duration::from_secs(1)) // 每 1 秒补充令牌
        .refill(1) // 每次补充 1 个令牌
        .build();
    
    // 将令牌桶封装在 Arc 和 Mutex 中以支持线程安全的共享
    let bucket = Arc::new(Mutex::new(bucket));


    // day12


    let board_grid: Arc<Mutex<Board> > = Arc::new(Mutex::new ( Board::new()));
    //let app_state: Arc<Mutex<AppState> > = Arc::new(Mutex::new ( AppState::new(2024)));

    // let mut board_grid: [ [char; 6] ; 5] = [
    // [ '⬜', '⬛', '⬛', '⬛', '⬛', '⬜' ],
    // [ '⬜', '⬛', '⬛', '⬛', '⬛', '⬜' ],
    // [ '⬜', '⬛', '⬛', '⬛', '⬛', '⬜' ],
    // [ '⬜', '⬛', '⬛', '⬛', '⬛', '⬜' ],
    // [ '⬜', '⬜', '⬜', '⬜', '⬜', '⬜' ],
    // ];



    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(crate::day_one::hello_bird)
            .service(crate::day_one::seek)
            .service(crate::day_two::dest)
            .service(crate::day_two::key)
            .service(crate::day_two::v6_dest)
            .service(crate::day_two::v6_key)
            .service(crate::day_five::handle_manifest)
            .app_data(web::Data::new(bucket.clone()))
            .service(crate::day_nine::handle_milk_request)
            .app_data(web::Data::new(board_grid.clone()))
            .service(crate::day_twelve::handle_board)
            .service(crate::day_twelve::board_reset)
            .service(crate::day_twelve::place_cookie_milk)
            .service(crate::day_twelve::random_board)
            .app_data(web::Data::new(board_grid.clone()))
            //.app_data(web::Data::new(app_state.clone()))
            .service(crate::day_sixteen::wrap_gift)
            .service(crate::day_sixteen::unwrap_gift)
            ;

    };

    Ok(config.into())
}
