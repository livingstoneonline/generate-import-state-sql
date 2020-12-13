use crate::Result;
use async_ftp::FtpStream;
use serde::Deserialize;
use std::io::{prelude::*, BufReader};

use std::{
    fs,
    fs::File,
    path::{Path, PathBuf},
};
use std::{str::FromStr, time::SystemTime};
use tokio_rustls::rustls::ClientConfig;
use tokio_rustls::webpki::DNSNameRef;

struct NoCertVerification;

// Do not validate the certificate client side.
impl tokio_rustls::rustls::ServerCertVerifier for NoCertVerification {
    fn verify_server_cert(
        &self,
        _: &rustls::RootCertStore,
        _: &[rustls::Certificate],
        _: tokio_rustls::webpki::DNSNameRef,
        _: &[u8],
    ) -> core::result::Result<rustls::ServerCertVerified, rustls::TLSError> {
        Ok(rustls::ServerCertVerified::assertion())
    }
}

const OBJECTS: &str = "import.objects.csv";
const DATASTREAMS: &str = "import.datastreams.csv";

async fn connect(server: &str, port: &str, user: &str, password: &str) -> Result<FtpStream> {
    let ftp_stream = FtpStream::connect(format!("{}:{}", server, port)).await?;
    // @todo Figure out how to fetch the full chain pem and load it into the certificate store.
    let mut conf = ClientConfig::new();
    conf.dangerous()
        .set_certificate_verifier(std::sync::Arc::new(NoCertVerification));
    let domain = DNSNameRef::try_from_ascii_str(server)?.into();
    // Switch to the secure mode (SSL).
    let mut ftp_stream = ftp_stream.into_secure(conf, domain).await?;
    ftp_stream.login(user, password).await?;
    Ok(ftp_stream)
}

async fn download(ftp_stream: &mut FtpStream, file: &str, dest: &Path) -> Result<()> {
    let download = ftp_stream.simple_retr(file).await?;
    File::create(dest)?.write_all(download.get_ref())?;
    Ok(())
}

async fn remote_size(ftp_stream: &mut FtpStream, file: &str) -> Result<u64> {
    Ok(ftp_stream.size(file).await?.unwrap() as u64)
}

async fn remote_date(ftp_stream: &mut FtpStream, file: &str) -> Result<SystemTime> {
    Ok(ftp_stream.mdtm(file).await?.unwrap().into())
}

fn local_size(path: &Path) -> Option<u64> {
    fs::metadata(path).map_or(None, |m| Some(m.len()))
}

fn local_date(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).map_or(None, |m| m.modified().map_or(None, |t| Some(t)))
}

async fn conditional_download(
    mut ftp_stream: &mut FtpStream,
    file: &str,
    dest: &str,
) -> Result<()> {
    let mut dest = PathBuf::from_str(dest)?;
    dest.push(file);
    let src_date = remote_date(&mut ftp_stream, file).await?;

    let different =
        if let (Some(dest_date), Some(dest_size)) = (local_date(&dest), local_size(&dest)) {
            let src_size = remote_size(&mut ftp_stream, file).await?;
            src_date != dest_date || src_size != dest_size
        } else {
            true
        };

    if different {
        download(&mut ftp_stream, file, &dest).await?;
        filetime::set_file_mtime(&dest, src_date.into())?;
    }
    Ok(())
}

async fn download_import_csv_files(
    server: &str,
    port: &str,
    user: &str,
    password: &str,
    src: &str,
    dest: &str,
) -> Result<()> {
    let mut ftp_stream = connect(server, port, user, password).await?;
    ftp_stream.cwd(src).await?;
    conditional_download(&mut ftp_stream, OBJECTS, &dest).await?;
    conditional_download(&mut ftp_stream, DATASTREAMS, &dest).await?;
    ftp_stream.quit().await.map_err(|e| e.into())
}

trait Rows {
    type Row: ToString + serde::de::DeserializeOwned;

    fn rows(path: &Path) -> std::result::Result<String, std::io::Error> {
        let mut rows = vec![];
        let file = File::open(path)?;
        let mut reader = csv::Reader::from_reader(BufReader::new(file));
        for result in reader.deserialize() {
            let item: Self::Row = result?;
            rows.push(item.to_string())
        }
        Ok(rows.join(",\n"))
    }

    fn sql(path: &Path) -> std::result::Result<String, std::io::Error>;
}

#[derive(Deserialize)]
struct Object {
    #[serde(alias = "PID")]
    pid: String,
    #[serde(alias = "CONTENT_MODEL")]
    content_model: String,
    #[serde(alias = "PRIVATE")]
    private: u32,
    #[serde(alias = "TYPE")]
    r#type: String,
    #[serde(alias = "MD5")]
    md5: String,
}

impl ToString for Object {
    fn to_string(&self) -> String {
        format!(
            "('{pid}', '{model}', {private}, '{type}', '{md5}')",
            pid = self.pid,
            model = self.content_model,
            private = self.private,
            r#type = self.r#type,
            md5 = self.md5
        )
    }
}

impl Rows for Object {
    type Row = Object;

    fn sql(path: &Path) -> std::result::Result<String, std::io::Error> {
        let rows = Object::rows(path)?;
        let sql = format!(
            r#"
TRUNCATE TABLE livingstone_fedora_remote_objects;
INSERT INTO livingstone_fedora_remote_objects (pid, content_model, private, type, md5)
VALUES
{};

"#,
            rows
        );
        Ok(sql)
    }
}

#[derive(Deserialize)]
struct Datastream {
    #[serde(alias = "PID")]
    pid: String,
    #[serde(alias = "DSID")]
    dsid: String,
    #[serde(alias = "MD5")]
    md5: String,
    #[serde(alias = "FILE")]
    file: String,
}

impl ToString for Datastream {
    fn to_string(&self) -> String {
        format!(
            "('{pid}', '{dsid}', '{md5}', '{file}')",
            pid = self.pid,
            dsid = self.dsid,
            md5 = self.md5,
            file = self.file,
        )
    }
}

impl Rows for Datastream {
    type Row = Datastream;

    fn sql(path: &Path) -> std::result::Result<String, std::io::Error> {
        let rows = Datastream::rows(path)?;
        let sql = format!(
            r#"
TRUNCATE TABLE livingstone_fedora_remote_datastreams;
INSERT INTO livingstone_fedora_remote_datastreams (pid, dsid, md5, file)
VALUES
{};

"#,
            rows
        );
        Ok(sql)
    }
}

pub async fn generate_sql(
    server: &str,
    port: &str,
    user: &str,
    password: &str,
    src: &str,
    dest: &str,
    skip: bool,
) -> Result<String> {
    if !skip {
        download_import_csv_files(server, port, user, password, src, dest).await?;
    }

    let mut object_file = PathBuf::from_str(dest)?;
    object_file.push(OBJECTS);
    let objects = async { Object::sql(&object_file) };

    let mut datastream_file = PathBuf::from_str(dest)?;
    datastream_file.push(DATASTREAMS);
    let datastreams = async { Datastream::sql(&datastream_file) };

    let (objects, datastreams) = tokio::try_join!(objects, datastreams)?;
    Ok(objects + datastreams.as_str())
}
