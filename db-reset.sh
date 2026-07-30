#!/bin/sh

set -e

cargo sqlx database reset -y
cargo run --bin seed
