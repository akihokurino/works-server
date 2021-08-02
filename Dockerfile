# alpineにするとbatchがなぜかこける
FROM rust:1.53.0 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM rust:1.53.0 AS deploy
WORKDIR /app
COPY --from=builder /app/target/release/app-api api
COPY --from=builder /app/target/release/app-batch batch
ENV TZ=Asia/Tokyo