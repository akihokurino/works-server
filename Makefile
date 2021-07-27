MAKEFLAGS=--no-builtin-rules --no-builtin-variables --always-make
ROOT := $(realpath $(dir $(lastword $(MAKEFILE_LIST))))
export PATH := $(ROOT)/scripts:$(PATH)

run-api:
	RUST_ENV=$(PWD)/.env.local \
	FIREBASE_CREDENTIALS=$(PWD)/firebase.prod.json \
	GOOGLE_APPLICATION_CREDENTIALS=$(PWD)/gcp.prod.json \
 	cargo run --bin app-api

run-batch-sync-invoice:
	RUST_ENV=$(PWD)/.env.local \
	FIREBASE_CREDENTIALS=$(PWD)/firebase.prod.json \
	GOOGLE_APPLICATION_CREDENTIALS=$(PWD)/gcp.prod.json \
 	cargo run --bin app-batch sync-invoice

build-api:
	cargo build --bin app-api

build-batch:
	cargo build --bin app-batch

clean:
	cargo clean

proxy_db:
	cloud_sql_proxy -credential_file=gcp.prod.json -instances=works-prod:asia-northeast1:main=tcp:0.0.0.0:3306

deploy:
	ENV=prod deploy_secret.sh
	ENV=prod deploy.sh

setup_infra:
	ENV=prod setup_infra.sh

deploy_oauth_callback:
	cd oauth/callback && firebase use production && firebase deploy