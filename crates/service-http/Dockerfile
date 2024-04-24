FROM rust:1.77.2 as build
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim AS runtime
WORKDIR /app
COPY --from=build /app/target/release/service-http .
CMD ["ls"]
ENTRYPOINT ["./service-http"]
