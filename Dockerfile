FROM rust:1.59.0-slim-buster AS build

RUN apt-get update \
    && apt-get install -y --no-install-recommends libssl-dev curl ca-certificates build-essential gfortran pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml /app/
COPY Cargo.lock /app/
COPY src/*.rs /app/src/

WORKDIR /app
RUN cargo build --release

#######################
FROM debian:buster-slim

COPY --from=build /app/target/release/web-feature /app

ENV ROCKET_ENV=prod
ENV ROCKET_PORT=80
ENV ROCKET_LOG=normal
EXPOSE 80

ENTRYPOINT ["/app"]
