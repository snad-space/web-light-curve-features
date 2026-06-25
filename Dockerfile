FROM rust:1.70.0-bookworm AS build

RUN apt-get update \
    && apt-get install -y --no-install-recommends libssl-dev curl ca-certificates build-essential gfortran pkg-config libceres-dev libfftw3-dev libgsl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml /app/
COPY Cargo.lock /app/
COPY src/*.rs /app/src/

WORKDIR /app
RUN cargo build --release --locked

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
# Keep idle upstream connections open longer than nginx-proxy's keepalive so the
# reverse proxy never reuses a connection Rocket already closed (avoids 502s).
# Rocket's default keep_alive is 5s; raise it to match the gunicorn fix in ztf-web.
ENV ROCKET_KEEP_ALIVE=75
EXPOSE 80

ENTRYPOINT ["/app"]
