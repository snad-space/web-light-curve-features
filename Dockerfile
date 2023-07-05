FROM rust:1.70.0-bookworm AS build

RUN apt-get update \
    && apt-get install -y --no-install-recommends libssl-dev curl ca-certificates build-essential gfortran pkg-config libceres-dev libfftw3-dev libgsl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml /app/
COPY Cargo.lock /app/
COPY src/*.rs /app/src/

WORKDIR /app
RUN cargo build --release

#######################
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends libceres3 libfftw3-bin libfftw3-double3 libgsl27 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /app/target/release/web-feature /app

ENV ROCKET_PROFILE=prod
ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=80
ENV ROCKET_LOG_LEVEL=normal
EXPOSE 80

ENTRYPOINT ["/app"]
