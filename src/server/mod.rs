pub mod database;
pub mod health;

use crate::config::Config;

use crate::db::dispatch::create_pool;
use axum::Router;
use sqlx::PgPool;
use std::net::{Ipv4Addr, SocketAddr};
use tokio::net::TcpListener;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(clap::Parser, Debug)]
pub struct ServeOpts {
    /// Use own database (to perform migrations)
    #[clap(long)]
    own_db: bool,
}

#[derive(OpenApi)]
#[openapi(
    info(title = "sqlxum API"),
    paths(
        database::do_query,
        database::get_users,
        database::add_user,
        database::update_user,
        database::delete_user,
        health::get_health,
        health::ping,
    ),
    components(
        schemas(
            database::QueryReq,
            database::UserRes,
            database::UserPostReq,
            database::UserPutReq,
            database::UserDeleteReq,
            database::UserDeleteRes,
            health::HealthStatus,
            health::Pong,
        ),
    ),
    tags(
        (name = "database", description = "Database"),
        (name = "health", description = "Basic service status"),
    )
)]
struct ApiDoc;

#[derive(Clone)]
pub(crate) struct AppState {
    pool: PgPool,
}

pub async fn launch(opts: &ServeOpts) -> anyhow::Result<()> {
    let config = Config::get()?;

    check_for_own_db(opts, &config)?;

    let pool = create_pool(&config).await?;

    if opts.own_db {
        sqlx::migrate!("./migrations").run(&pool).await?;
    }

    let app_state = AppState { pool: pool.clone() };

    let app = Router::new()
        .nest(
            "/api",
            Router::new()
                .merge(health::create_router())
                .merge(database::create_router(app_state).await?),
        )
        .merge(create_swagger_router(&config));

    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, config.port));
    let listener = TcpListener::bind(address.to_string()).await?;
    println!("Server listening on {}", address);

    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

fn check_for_own_db(opts: &ServeOpts, config: &Config) -> anyhow::Result<()> {
    if opts.own_db && !config.database_url.ends_with("/sqlxum_test") {
        return Err(anyhow::anyhow!(
            "Database name must be 'sqlxum_test' when using the --own-db option"
        ));
    }
    Ok(())
}

/// Adapted from odss2dash
fn create_swagger_router(config: &Config) -> SwaggerUi {
    let api_url = format!("{}/api", config.external_url);

    let mut doc = ApiDoc::openapi();
    doc.servers = Some(vec![utoipa::openapi::Server::new(&api_url)]);

    // For appropriate dispatch of SwaggerUI on deployed site:

    // (a) this value is good for both local and deployed site:
    let apidoc_rel = "/apidoc";

    let json_rel = if config.external_url.ends_with("/sqlxum") {
        // (b) for deployed site, need to prefix with /sqlxum/
        // per proxy setting on target server:
        "/sqlxum/api-docs/openapi.json"
    } else {
        "/api-docs/openapi.json"
    };

    // (c) use the value in (b) for Config::from(), so that the correct url
    // is used by swagger-ui app (setting in swagger-initializer.js):
    let swagger_ui_config = utoipa_swagger_ui::Config::from(json_rel)
        .display_operation_id(true)
        .use_base_layout();

    let swagger_ui = SwaggerUi::new(apidoc_rel)
        // (d) as with (a), value here is good in general:
        .url("/api-docs/openapi.json", doc)
        .config(swagger_ui_config);

    println!("api : {}", &api_url);
    println!("doc : {}/apidoc/", config.external_url);
    println!("spec: {}/api-docs/openapi.json", config.external_url);

    swagger_ui
}

/// Slightly adapted from realworld-axum-sqlx
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Should install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Should install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            println!("\nBye!");
        },
        _ = terminate => {
            println!("\nTerminate signal received, shutting down");
        },
    }
}
