
use actix_web::{middleware::Logger,  web::ServiceConfig, App, };

use shuttle_actix_web::ShuttleActixWeb;



mod day_one;
pub use crate::day_one::{hello_bird, seek};

mod day_two;
pub use crate::day_two::{dest, key, v6_dest, v6_key};


mod day_five;
pub use crate::day_five::handle_manifest;




#[shuttle_runtime::main]
async fn main() -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    //env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    env_logger::Env::new().default_filter_or("info");

    let _app = App::new()
        //.wrap(Logger::new("%a %{User-Agent}i"));
        .wrap(Logger::default());

    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(crate::day_one::hello_bird)
            .service(crate::day_one::seek)
            .service(crate::day_two::dest)
            .service(crate::day_two::key)
            .service(crate::day_two::v6_dest)
            .service(crate::day_two::v6_key)
            .service(handle_manifest);
    };

    Ok(config.into())
}
