# syntax=docker/dockerfile:1
ARG RUST_VERSION=1.99
FROM rust:${RUST_VERSION}-slim AS build
WORKDIR /src
COPY --chown=build:build Cargo.toml Cargo.lock ./
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release && strip target/release/ive

RUN <<EOF
apt-get update
apt-get install -y ca-certificates
EOF

# Runtime image
FROM debian:bookworm-slim@sha256:0a1b2c3d4e5f
LABEL org.opencontainers.image.title="IVE \"editor\"" maintainer='team@example.com'
ENV APP_HOME=/opt/ive \
    PATH="/opt/ive/bin:$PATH"
COPY --from=build /src/target/release/ive $APP_HOME/bin/
COPY <<EOF /etc/ive/ive.conf
port = 8080
EOF
EXPOSE 8080/tcp
USER ive:ive
VOLUME ["/data"]
HEALTHCHECK --interval=30s CMD curl -f http://localhost:8080/health || exit 1
ENTRYPOINT ["ive", "--serve"]
CMD ["--port", "8080"]
