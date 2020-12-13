#!/usr/bin/env bash

set -e

readonly PROGNAME=$(basename $0)
readonly PROGDIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly ARGS="$@"

function main() {
    local package_directory=${PROGDIR}/package
    mkdir -p ${package_directory} 2>/dev/null
    cargo build --release
    cp ${PROGDIR}/target/x86_64-unknown-linux-musl/release/generate-import-state-sql ${package_directory}
    strip ${package_directory}/generate-import-state-sql
}
main
