FROM debian:buster AS build

ARG TOOLCHAIN=nightly-2020-02-18

RUN apt-get update \
    && apt-get install -y --no-install-recommends curl ca-certificates gcc libc-dev \
    && rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain ${TOOLCHAIN}

COPY Cargo.toml /app/
COPY Cargo.lock /app/
COPY src/*.rs /app/src/

WORKDIR /app
RUN $HOME/.cargo/bin/cargo build --release

###################
FROM debian:buster

COPY --from=build /app/target/release/web-feature /app

ENV ROCKET_ENV=prod
ENV ROCKET_PORT=80
ENV ROCKET_LOG=normal
EXPOSE 80

ENTRYPOINT ["/app"]
