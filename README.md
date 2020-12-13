# Generate Import State SQL <!-- omit in toc -->

[![LICENSE](https://img.shields.io/badge/license-GPL3-blue.svg?style=flat-square)](./LICENSE)

- [Introduction](#introduction)
- [Usage](#usage)
- [Building](#building)
- [Packaging](#packaging)

## Introduction

This is a CLI tool used to generates an SQL file which will contains the current
state of the Livingstone Online FTP server and the Livingstone Online Fedora
Repository. This information is then used to decided which content requires
updating in the Livingstone Online Fedora repository.

## Usage

```bash
Generates SQL file to update local/remote tables.

USAGE:
    generate-import-state-sql [OPTIONS] --ftp-dest <ftp-dest> --ftp-password <ftp-password> --ftp-port <ftp-port> --ftp-server <ftp-server> --ftp-src <ftp-src> --ftp-user <ftp-user> --solr <solr> --sql <sql>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --ftp-dest <ftp-dest>            The local folder to download the CSV files to.
        --ftp-password <ftp-password>    Use password to authenticate against the FTP server.
        --ftp-port <ftp-port>            Port to connect to the FTP server with.
        --ftp-server <ftp-server>        FTP server to connect to and fetch the import CSV files from.
        --ftp-skip <ftp-skip>            Skips downloading import CSV files if already present. [default: false]
        --ftp-src <ftp-src>              The folder on the FTP server that contains the import CSV files.
        --ftp-user <ftp-user>            Connect to FTP server as user.
        --solr <solr>                    The URL to solr which will be queried for local information.
        --sql <sql>                      The full path to write the SQL file to.
```

For this to work the Solr query needs to limit the set of CModels to only those which we support importing:

- info:fedora/islandora:manuscriptCModel
- info:fedora/islandora:manuscriptPageCModel
- info:fedora/islandora:sp_large_image_cmodel
- info:fedora/islandora:sp_pdf
- info:fedora/livingstone:spectralManuscriptCModel
- info:fedora/livingstone:spectralManuscriptPageCModel

For example:

```bash
tomcat:8080/solr/collection1/select?q=RELS_EXT_hasModel_uri_s%3A%22info%3Afedora%2Fislandora%3AmanuscriptCModel%22%2C%0ARELS_EXT_hasModel_uri_s%3A%22info%3Afedora%2Fislandora%3AmanuscriptPageCModel%22%2C%0ARELS_EXT_hasModel_uri_s%3A%22info%3Afedora%2Fislandora%3Asp_large_image_cmodel%22%0ARELS_EXT_hasModel_uri_s%3A%22info%3Afedora%2Fislandora%3Asp_pdf%22%2C%0ARELS_EXT_hasModel_uri_s%3A%22info%3Afedora%2Flivingstone%3AspectralManuscriptCModel%22%2C%0ARELS_EXT_hasModel_uri_s%3A%22info%3Afedora%2Flivingstone%3AspectralManuscriptPageCModel%22&sort=PID+asc&rows=100000&fl=PID%2CRELS_EXT_hasModel_uri_s%2Cchecksum_s%2Chidden_b%2Cfedora_datastream_latest_*_MD5_ms&wt=xml&indent=true
```

## Building

Building the tool requires a local installation of [Rust]. Instructions for
installing [Rust] can be found
[here](https://www.rust-lang.org/learn/get-started).

This tool is expected to be run inside of a [Alpine](https://alpinelinux.org/)
container so it must target [musl](https://www.musl-libc.org/).

This requires the `x86_64-unknown-linux-musl` target to be installed for via `rustup`:

```bash
rustup target add x86_64-unknown-linux-musl
```

To build locally simply use the following `cargo` commands.

**Debug Build**:

```bash
cargo build
```

**Release Build**:

```bash
cargo build --release
```

**Execute Tests**:

```bash
cargo test
```

**Run Debug**:

```bash
cargo run
```

**Run Release**:

```bash
cargo run --release
```

## Packaging

For the sake of convenience a small script has been bundled to package the
application.

```bash
./package.sh
```
