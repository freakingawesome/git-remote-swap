FROM rust:1.49 as builder

RUN USER=root cargo new --bin git-remote-swap
WORKDIR ./git-remote-swap

# This first section is a caching mechanism of dependencies.
COPY ./Cargo.toml ./Cargo.toml
RUN mkdir -p src/lib && touch src/lib/lib.rs
RUN cargo build --release
RUN rm src/*.rs

ADD . ./

RUN rm ./target/release/deps/*
RUN cargo test
RUN cargo build --release

FROM debian:buster-slim
ARG APP=/usr/src/app

RUN apt-get update \
    && apt-get install -y openssl \
    && rm -rf /var/lib/apt/lists/*

ENV APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=builder /git-remote-swap/target/release/git-remote-swap ${APP}/git-remote-swap

RUN chown -R $APP_USER:$APP_USER ${APP}

# Use a dummy config file so it doesn't error out
RUN mkdir -p /app/config && echo "remotes: []" > /app/config/git-remote-swap.yaml

VOLUME ["/app/config", "/app/root"]

USER $APP_USER
WORKDIR ${APP}

ENTRYPOINT ["./git-remote-swap"]
CMD ["--config", "/app/config/git-remote-swap.yaml", "--root", "/app/root"]
