# (1) installing cargo-chef & build deps
FROM rust:alpine AS chef
WORKDIR /app
RUN cargo install --locked cargo-chef
RUN apk update && apk add --no-cache upx curl musl-dev openssl\
    openssl-dev pkgconfig gcc openssl-libs-static

# (2) Installing rust target for alpine

# (3) prepating recipe file
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# (4) building project deps, cache magic happen on COPY command
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --recipe-path recipe.json --release

# (5) building binary
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

# (6) compress binary. Compress the binary. Use the command 'upx -t ./target/release/server'
#     to verify the compression.
RUN upx --ultra-brute -v /app/target/x86_64-unknown-linux-musl/release/server &&\
    upx -t /app/target/x86_64-unknown-linux-musl/release/server

# (7) runtime image based on alpine
FROM alpine:latest AS runtime
# RUN apk add --no-cache curl busybox-extras bash
RUN addgroup -g 1001 app && adduser -u 1001 -G app -S app

WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/server /app/server
COPY --from=builder /app/assets/config.yaml* /app/config.yaml
COPY --from=builder /app/entrypoint.sh /app/entrypoint.sh

RUN chmod +x /app/entrypoint.sh
RUN chown -R app:app /app
USER app:app

ENTRYPOINT ["/app/entrypoint.sh"]