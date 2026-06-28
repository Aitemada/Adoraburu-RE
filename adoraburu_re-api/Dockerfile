FROM rust:1.95-slim-bookworm as rs-Api-Buildstage

WORKDIR /api

COPY Cargo.toml Cargo.lock ./
RUN apt-get update && apt-get install -y pkg-config libssl-dev && mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf /var/lib/apt/lists/*
# ^Usin' --release flag so it builds faster. Simple as cheese. 

COPY src ./src
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim
WORKDIR /api

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=rs-Api-Buildstage /api/target/release/adoraburu_re-api /api/adoraburu_re-api

EXPOSE 3111
CMD ["./adoraburu_re-api"]