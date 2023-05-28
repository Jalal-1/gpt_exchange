use dotenv::dotenv;
use gpt_exchange::data::AppDatabase;
use gpt_exchange::domain::maintenance::Maintenance;
use gpt_exchange::web::{renderer::Renderer, responsecounter::ResponseCounter};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "httpd")]
struct Opt {
    #[structopt(default_value = "sqlite:data.db")]
    connection_string: String,
    #[structopt(short, long, parse(from_os_str), default_value = "templates/")]
    template_directory: PathBuf,
}

fn main() {
    dotenv().ok();
    let opt = Opt::from_args();

    let rt = tokio::runtime::Runtime::new().expect("failed to spawn tokio runtime");

    let handle = rt.handle().clone();
    let renderer = Renderer::new(opt.template_directory.clone());

    let database = rt.block_on(async move { AppDatabase::new(&opt.connection_string).await });

    let hit_counter = ResponseCounter::new(database.get_pool().clone(), handle.clone());
    let maintenance = Maintenance::spawn(database.get_pool().clone(), handle);

    let config = gpt_exchange::RocketConfig {
        renderer,
        database,
        response_counter: hit_counter,
        maintenance,
    };

    rt.block_on(async move {
        gpt_exchange::rocket(config)
            .launch()
            .await
            .expect("failed to launch rocket server")
    });
}
