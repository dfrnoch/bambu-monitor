FROM rust:1-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --system --uid 10001 --create-home bambu

WORKDIR /app
COPY --from=builder /app/target/release/bambu-monitor /usr/local/bin/bambu-monitor
COPY assets ./assets

ENV HOST=0.0.0.0
ENV PORT=3000
ENV DATA_DIR=/data
ENV AUTO_CONNECT=true

RUN mkdir -p /data && chown -R bambu:bambu /data

USER bambu
EXPOSE 3000

CMD ["bambu-monitor"]
