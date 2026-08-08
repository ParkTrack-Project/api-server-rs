FROM rust:1 AS build-env
WORKDIR /app
COPY Cargo.lock Cargo.toml /app/
COPY src /app/src/
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
COPY --from=build-env /app/target/release/api-server-rust /
CMD ["./api-server-rust"]