//! Main application library

pub mod data;
pub mod domain;
pub mod service;
pub mod web;

pub use data::DataError;
pub use domain::job::field::ShortCode;
pub use domain::job::{Job, JobError};
pub use domain::time::Time;
pub use service::ServiceError;

use data::AppDatabase;
use domain::maintenance::Maintenance;
use rocket::fs::FileServer;
use rocket::{Build, Rocket};
use web::renderer::Renderer;
use web::responsecounter::ResponseCounter;

/// Creates a new Rocket build that is configured for running JobStash.
pub fn rocket(config: RocketConfig) -> Rocket<Build> {
    rocket::build()
        .manage::<AppDatabase>(config.database)
        .manage::<Renderer>(config.renderer)
        .manage::<ResponseCounter>(config.response_counter)
        .manage::<Maintenance>(config.maintenance)
        .mount("/", web::http::routes())
        .mount("/api/job", web::api::routes())
        .mount("/static", FileServer::from("static"))
        .register("/", web::http::catcher::catchers())
        .register("/api/job", web::api::catcher::catchers())
}

/// Data needed to set up exchange with Rocket.
pub struct RocketConfig {
    pub renderer: Renderer<'static>,
    pub database: AppDatabase,
    pub response_counter: ResponseCounter,
    pub maintenance: Maintenance,
}

#[cfg(test)]
pub mod test {
    pub fn async_runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .max_blocking_threads(1)
            .enable_time()
            .build()
            .expect("failed to spawn tokio runtime")
    }
}
