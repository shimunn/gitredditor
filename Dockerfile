FROM scratch AS fs

COPY . /usr/src/gitredditor

FROM rust:1.35 AS build

WORKDIR /usr/src/

ENV USER root

RUN cargo new --lib gitredditor

WORKDIR /usr/src/gitredditor

COPY --from=fs /usr/src/gitredditor/Cargo.toml /usr/src/gitredditor/Cargo.lock ./

RUN cargo build --release

COPY --from=fs /usr/src/gitredditor/src src

RUN cargo build --release

FROM ubuntu


RUN apt update && apt install -y libcurl3 git gitstats -y && rm -rf /var/lib/{apt,dpkg,cache,log}

COPY --from=build /usr/src/gitredditor/target/release/gitredditor /usr/local/bin

COPY --from=fs /usr/src/gitredditor/src /usr/src/gitredditor/src

COPY --from=fs /usr/src/gitredditor/stats.sh /usr/local/bin/gitredditor-stats

RUN chmod +x /usr/local/bin/gitredditor-stats

VOLUME /repo /stats

WORKDIR /repo

ENTRYPOINT gitredditor

CMD ["-r", "$REDDIT_USERNAME", "/repo"]
