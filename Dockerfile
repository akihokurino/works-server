FROM rust:1.54.0-alpine3.13 AS builder
WORKDIR /app
COPY . .
RUN apk update \
    && apk add --no-cache musl-dev mariadb-dev \
    && cargo build --release


FROM alpine AS deploy
WORKDIR /app
RUN apk add --no-cache ca-certificates bash
COPY --from=builder /app/target/release/app-api api
COPY --from=builder /app/target/release/app-batch batch
ENV TZ=Asia/Tokyo