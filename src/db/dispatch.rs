use sqlx::postgres::PgPoolOptions;

use crate::config::Config;
use crate::db::generic::do_query;
use crate::db::DbOpts;

pub(crate) async fn dispatch(opts: &DbOpts) -> anyhow::Result<()> {
    let config = Config::get()?;
    let pool = create_pool(&config).await?;

    if let Some(query) = &opts.query {
        let res = do_query(&pool, query).await?;
        println!("{}", serde_json::to_string_pretty(&res)?);
    }
    Ok(())
}

pub async fn create_pool(config: &Config) -> sqlx::Result<sqlx::PgPool> {
    log::info!("Connecting to database...");
    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(9))
        .connect(&config.database_url)
        .await
}
