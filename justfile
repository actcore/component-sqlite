wasm := "target/wasm32-wasip2/release/component_sqlite.wasm"
act := env("ACT", "act")
port := "3456"
addr := "[::1]:" + port
baseurl := "http://" + addr

build:
    CC=/opt/wasi-sdk/bin/clang cargo build --target wasm32-wasip2 --release

test:
    #!/usr/bin/env bash
    {{act}} serve {{wasm}} --listen "{{addr}}" &
    trap "kill $!" EXIT
    npx wait-on {{baseurl}}/info
    hurl --test --variable "baseurl={{baseurl}}" e2e/*.hurl
