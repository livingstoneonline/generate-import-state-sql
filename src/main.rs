mod args;
mod ftp;
mod solr;

use args::*;
use std::fs::File;
use std::io::Write;
use tokio::runtime::Builder;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[allow(clippy::too_many_arguments)]
async fn generate_sql(
    ftp_server: &str,
    ftp_port: &str,
    ftp_user: &str,
    ftp_password: &str,
    ftp_src: &str,
    ftp_dest: &str,
    ftp_skip: bool,
    solr: &str,
    sql: &str,
) -> Result<()> {
    let ftp_sql = ftp::generate_sql(
        ftp_server,
        ftp_port,
        ftp_user,
        ftp_password,
        ftp_src,
        ftp_dest,
        ftp_skip,
    );
    let solr_sql = solr::generate_sql(solr);
    let (ftp_sql, solr_sql) = tokio::try_join!(ftp_sql, solr_sql)?;
    let mut file = File::create(sql)?;
    file.write_all(ftp_sql.as_bytes())?;
    file.write_all(solr_sql.as_bytes())?;
    Ok(())
}

fn main() -> Result<()> {
    let mut args = args();
    let matches = args.clone().get_matches();
    if matches.is_present("help") {
        args.print_long_help().unwrap();
        Ok(())
    } else {
        let ftp_server = matches.value_of("ftp-server").unwrap();
        let ftp_port = matches.value_of("ftp-port").unwrap();
        let ftp_user = matches.value_of("ftp-user").unwrap();
        let ftp_password = matches.value_of("ftp-password").unwrap();
        let ftp_src = matches.value_of("ftp-src").unwrap();
        let ftp_dest = matches.value_of("ftp-dest").unwrap();
        let ftp_skip: bool = matches.value_of("ftp-skip").unwrap().parse()?;
        let solr = matches.value_of("solr").unwrap();
        let sql = matches.value_of("sql").unwrap();
        let mut runtime = Builder::new()
            .threaded_scheduler()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(generate_sql(
            ftp_server,
            ftp_port,
            ftp_user,
            ftp_password,
            ftp_src,
            ftp_dest,
            ftp_skip,
            solr,
            sql,
        ))
    }
}
