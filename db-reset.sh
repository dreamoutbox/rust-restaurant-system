#!/bin/sh

cargo sqlx database reset -y
cargo run --bin seed
