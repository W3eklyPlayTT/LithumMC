FROM rust:latest AS build

WORKDIR /build
COPY . .
RUN cargo build --locked --release --package lithummc --bin lithummc-server

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --gid 2613 lithummc \
    && useradd --uid 2613 --gid lithummc --create-home --home-dir /lithummc lithummc

COPY --from=build /build/target/release/lithummc-server /usr/local/bin/lithummc-server
WORKDIR /lithummc
USER lithummc:lithummc

ENV RUST_BACKTRACE=1
EXPOSE 25565/tcp
EXPOSE 19132/udp

ENTRYPOINT ["/usr/local/bin/lithummc-server"]
