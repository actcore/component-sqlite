wasm := "target/wasm32-wasip2/release/component_sqlite.wasm"
act := env("ACT", "act")
port := `python3 -c 'import socket; s=socket.socket(socket.AF_INET, socket.SOCK_STREAM); s.bind(("", 0)); print(s.getsockname()[1]); s.close()'`
addr := "[::1]:" + port
baseurl := "http://" + addr

build:
    CC=/opt/wasi-sdk/bin/clang cargo build --target wasm32-wasip2 --release

test:
    #!/usr/bin/env bash
    DB_DIR=$(mktemp -d)
    DB_PATH="$DB_DIR/test.db"
    {{act}} serve {{wasm}} --listen "{{addr}}" --allow-dir "/data:$DB_DIR" &
    trap "kill $!; rm -rf $DB_DIR" EXIT
    npx wait-on {{baseurl}}/info
    hurl --test --variable "baseurl={{baseurl}}" --variable "db_path=/data/test.db" e2e/*.hurl
