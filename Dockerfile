FROM scratch AS fs

COPY . /usr/src/gitredditor

FROM rust:1.34.1 AS build

WORKDIR /usr/src/

ENV USER root

RUN cargo new --lib gitredditor

WORKDIR /usr/src/gitredditor

COPY --from=fs /usr/src/gitredditor/Cargo.toml /usr/src/gitredditor/Cargo.lock ./

RUN cargo build --release

COPY --from=fs /usr/src/gitredditor/src src

RUN cargo build --release

FROM ubuntu


RUN apt update && apt install -y libcurl3 git -y && rm -rf /var/lib/{apt,dpkg,cache,log}

COPY --from=build /usr/src/gitredditor/target/release/gitredditor /usr/local/bin

COPY --from=fs /usr/src/gitredditor/src /usr/src/gitredditor/src

VOLUME /repo

WORKDIR /repo

ENTRYPOINT gitredditor

CMD ["-r", "$REDDIT_USERNAME", "/repo"]
