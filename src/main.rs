use actix_web::{web, middleware::Logger,  web::ServiceConfig, App, };
use actix_files::Files;

use shuttle_actix_web::ShuttleActixWeb;


mod day_one;

mod day_two;


mod day_five;


mod day_nine;
use leaky_bucket::RateLimiter;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Duration;



mod day_twelve;

pub use day_twelve:: Board;


mod day_sixteen;


mod day_nineteen;
use deadpool_postgres::{Manager, Pool, PoolError};



use sqlx::PgPool;
use crate::day_nineteen::{reset, get_quote, delete_quote, update_quote, add_quote};

mod day_twentythree;
pub use day_twentythree::OrnamentState;


#[shuttle_runtime::main]
//#[shuttle_service::main]
async fn main(#[shuttle_shared_db::Postgres] pool:PgPool) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    //env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    env_logger::Env::new().default_filter_or("info");

    // let _app = App::new()
    //     //.wrap(Logger::new("%a %{User-Agent}i"));
    //     .wrap(Logger::default());

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

 
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");


    //let pool = PgPool::connect(database_url).await?;

    let pool_data = web::Data::new(pool);


    let ornament_state = Arc::new(Mutex::new("off".to_string()));



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
            .service(crate::day_nine::handle_milk_refill)
            .app_data(web::Data::new(board_grid.clone()))
            .service(crate::day_twelve::handle_board)
            .service(crate::day_twelve::board_reset)
            .service(crate::day_twelve::place_cookie_milk)
            .service(crate::day_twelve::random_board)
            .app_data(web::Data::new(board_grid.clone()))
            //.app_data(web::Data::new(app_state.clone()))
            .service(crate::day_sixteen::wrap_gift)
            .service(crate::day_sixteen::unwrap_gift)
            .service(crate::day_sixteen::decode_jwt)
            .app_data(pool_data.clone())
            .service(crate::day_nineteen::reset)
            .service(crate::day_nineteen::get_quote)
            .service(crate::day_nineteen::delete_quote)
            .service(crate::day_nineteen::update_quote)
            .service(crate::day_nineteen::add_quote)
            .service(Files::new("/assets", "./assets"))
            .service(crate::day_twentythree::light_star)
            .service(crate::day_twentythree::present_color)
            .app_data(web::Data::new(OrnamentState {
                state: ornament_state.clone(),
            }))
            .service(crate::day_twentythree::ornament_color)
            ;

    };

    Ok(config.into())
}
