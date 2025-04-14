FROM rust:1.86.0-bullseye AS build
WORKDIR /build
RUN mkdir ./src
COPY Cargo.toml Cargo.lock . 
RUN echo 'fn main() {}' >> ./src/main.rs 
RUN cargo fetch --locked
COPY ./assets/ ./assets/
COPY ./src/ ./src/
RUN cargo build --release
ENTRYPOINT ["false"]

FROM debian:bullseye AS release
RUN apt-get update && apt-get install ca-certificates
WORKDIR /app
COPY --from=build /build/target/release/discord-gacha ./
ENTRYPOINT ["./discord-gacha"]
