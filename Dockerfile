# lagn: production image and Linux QA image.
#
#   docker build -t lagn .                       # production server
#   docker build --target qa -t lagn-qa .        # full QA suite on Linux
#
# Multi-arch: add --platform linux/amd64 or linux/arm64.

# --- web app -----------------------------------------------------------------
FROM node:22-trixie AS web
WORKDIR /src/web
COPY web/package.json web/package-lock.json ./
RUN npm ci --no-audit --no-fund
COPY web/ ./
RUN npm run build

# --- Rust build ------------------------------------------------------------------
FROM rust:1.98-trixie AS build
WORKDIR /src
COPY . .
RUN cargo build --release --locked -p lagn-server -p lagn-cli

# --- QA on Linux -----------------------------------------------------------------
FROM build AS qa
RUN apt-get update && apt-get install -y --no-install-recommends python3 curl && rm -rf /var/lib/apt/lists/*
CMD ["bash", "scripts/qa_linux.sh"]

# --- production runtime --------------------------------------------------------
FROM debian:trixie-slim AS runtime
# tzdata gives the server an OS copy of the tz database as well; the newest of
# that, the shipped data/zoneinfo and jiff's bundled copy is used.
RUN apt-get update && apt-get install -y --no-install-recommends tzdata ca-certificates curl \
 && rm -rf /var/lib/apt/lists/* && useradd --system --uid 10001 lagn
WORKDIR /app
COPY --from=build /src/target/release/lagn-server /src/target/release/lagn /usr/local/bin/
COPY ephe/ ./ephe/
COPY corpus/ ./corpus/
COPY data/ ./data/
COPY --from=web /src/web/dist ./web/dist
USER lagn
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s CMD curl -fsS http://127.0.0.1:8080/api/health || exit 1
# Review mode stays disabled unless a token file is mounted and passed with
# --review-token-file.
ENTRYPOINT ["lagn-server", "--addr", "0.0.0.0:8080", "--static", "/app/web/dist", \
            "--ephe", "/app/ephe", "--corpus", "/app/corpus", "--places", "/app/data/places.tsv", \
            "--tzdb", "/app/data/zoneinfo"]
