FROM rust:1-slim-bookworm AS build-env
WORKDIR /app
COPY Cargo.lock Cargo.toml /app/
COPY src /app/src/
RUN --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/var/cache/cargo \
    CARGO_HOME=/var/cache/cargo cargo build --release && \
    cp /app/target/release/api-server-rust /app/api-server-rust

FROM gcr.io/distroless/cc-debian12
COPY --from=build-env /app/api-server-rust /
CMD ["./api-server-rust"]