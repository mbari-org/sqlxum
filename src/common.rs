/// Replace backticks with single quotes, to facilitate passing in queries
/// from command line, e.g.:
/// ```shell
///  just query 'select * from foo where date >= `20240214` limit 3'
/// ```
pub fn unescape_query(query: &str) -> String {
    query.replace('`', "'")
}
