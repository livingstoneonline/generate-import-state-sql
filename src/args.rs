extern crate clap;

use clap::{App, Arg};
use std::env;
use std::ffi::OsStr;
use std::path::Path;

pub fn args<'a, 'b>() -> App<'a, 'b> {
    let args: Vec<String> = env::args().collect();
    let program_name = Path::new(OsStr::new(&args[0]))
        .file_name()
        .expect("Failed to get program name.");
    let program_name = program_name.to_string_lossy();
    App::new(program_name)
        .version("1.0")
        .author("Nigel Banks <nigel.g.banks@gmail.com>")
        .about("Generates SQL file to update local/remote tables.")
        .arg(
            Arg::with_name("ftp-server")
                .help("FTP server to connect to and fetch the import CSV files from.")
                .long("ftp-server")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("ftp-port")
                .help("Port to connect to the FTP server with.")
                .long("ftp-port")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("ftp-user")
                .help("Connect to FTP server as user.")
                .long("ftp-user")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("ftp-password")
                .help("Use password to authenticate against the FTP server.")
                .long("ftp-password")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("ftp-src")
                .help("The folder on the FTP server that contains the import CSV files.")
                .long("ftp-src")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("ftp-dest")
                .help("The local folder to download the CSV files to.")
                .long("ftp-dest")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("ftp-skip")
                .help("Skips downloading import CSV files if already present.")
                .long("ftp-skip")
                .default_value("false")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("solr")
                .help("The URL to solr which will be queried for local information.")
                .long("solr")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("sql")
                .help("The full path to write the SQL file to.")
                .long("sql")
                .required(true)
                .takes_value(true),
        )
}
