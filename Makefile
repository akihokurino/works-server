MAKEFLAGS=--no-builtin-rules --no-builtin-variables --always-make
ROOT := $(realpath $(dir $(lastword $(MAKEFILE_LIST))))
export PATH := $(ROOT)/scripts:$(PATH)

run-local:
	cargo run

build:
	cargo build

clean:
	cargo clean

proxy_db:
	cloud_sql_proxy -credential_file=works-cred.prod.json -instances=works-prod:asia-northeast1:db=tcp:0.0.0.0:3306