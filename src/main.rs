use crate::db::DbOpts;
use crate::server::ServeOpts;
use clap::Parser;

mod common;
mod config;
mod db;
mod models;
mod server;

#[derive(clap::Parser, Debug)]
#[clap(
    version,
    about = "sqlxum playground",
    long_about = "Welcome to some sqlx+axum exploration!"
)]
enum Opts {
    /// Launch server
    Serve(ServeOpts),

    /// Database related actions
    Db(DbOpts),

    /// Get health status
    Health,
}

#[tokio::main]
async fn main() {
    init();
    match Opts::parse() {
        Opts::Serve(opts) => serve(&opts).await,
        Opts::Db(opts) => dispatch(&opts).await,
        Opts::Health => health(),
    };
}

fn init() {
    dotenvy::dotenv().ok();
    env_logger::init();
}

async fn serve(opts: &ServeOpts) {
    match server::launch(opts).await {
        Ok(_) => (),
        Err(e) => log::error!("Error launching server: {}", e),
    }
}

async fn dispatch(opts: &DbOpts) {
    match db::dispatch::dispatch(opts).await {
        Ok(_) => (),
        Err(e) => log::error!("Error dispatching db command: {}", e),
    }
}

fn health() {
    let status = server::health::get_health_status();
    match serde_json::to_string_pretty(&status) {
        Ok(json) => println!("{}", json),
        Err(e) => {
            log::error!("Error serializing health status: {}", e);
            println!("{:?}", status);
        }
    }
}
