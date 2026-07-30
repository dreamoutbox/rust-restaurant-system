#!/bin/bash

# Terminate all background jobs when exiting or pressing Ctrl+C
trap 'kill $(jobs -p) 2>/dev/null' EXIT INT TERM

cargo watch -w src -x "run --bin rust-restaurant-system" &
pnpm --prefix web dev &

wait
