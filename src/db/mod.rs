pub(crate) mod dispatch;
pub(crate) mod generic;
pub(crate) mod users;

#[derive(clap::Parser, Debug)]
pub struct DbOpts {
    /// Use own database (to perform migrations)
    #[clap(long)]
    own_db: bool,

    /// Run query
    #[clap(long, value_name = "QUERY")]
    query: Option<String>,
}
