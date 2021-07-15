FROM rust:1.53.0 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM alpine as deploy
WORKDIR /app
EXPOSE 80
RUN apk add ca-certificates
COPY --from=builder /app/target/release/works-server /app/main
ENV TZ=Asia/Tokyo