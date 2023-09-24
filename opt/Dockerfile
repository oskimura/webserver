FROM rust:1-slim-buster as builder

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
    default-libmysqlclient-dev mariadb-client \
    && apt-get -y clean \
    && rm -rf /var/lib/apt/lists/*

# WORKDIR /usr/src/myapp

WORKDIR /usr/src/
COPY . .
#COPY ./practice-rust ./practice-rust
#RUN ls
COPY ./practice-rust .
WORKDIR /usr/src/practice-rust
#COPY . .
#RUN ls
RUN rustup component add rustfmt
RUN cargo install --path .

FROM debian:10.13-slim as stage

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
    default-libmysqlclient-dev mariadb-client \
    && apt-get -y clean \
    && rm -rf /var/lib/apt/lists/*

EXPOSE 8080
# COPY /usr/local/cargo/bin/practice-rust /usr/local/bin/practice-rust
COPY --from=builder /usr/local/cargo/bin/practice-rust /usr/local/bin/practice-rust

CMD [ "practice-rust" ]
