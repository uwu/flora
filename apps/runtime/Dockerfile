# syntax=docker/dockerfile:1.7

# ---------------------------------------------------------------------------
# Stage 1: Build the frontend
# ---------------------------------------------------------------------------
FROM node:22-slim AS frontend

WORKDIR /app
RUN apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates \
  && rm -rf /var/lib/apt/lists/* \
  && corepack enable

# Copy workspace root manifests
COPY package.json pnpm-workspace.yaml pnpm-lock.yaml ./
COPY tsconfig.json ./

# Copy all workspace package.json files so pnpm lockfile resolves
COPY packages/api-client/package.json packages/api-client/
COPY packages/branding/package.json packages/branding/
COPY packages/cli/package.json packages/cli/
COPY packages/sdk/package.json packages/sdk/
COPY apps/build-service/package.json apps/build-service/
COPY apps/frontend/package.json apps/frontend/
COPY apps/frontend-new/package.json apps/frontend-new/
COPY apps/uwu.network/package.json apps/uwu.network/
COPY apps/www/package.json apps/www/
COPY examples/basic/package.json examples/basic/

# Install deps (frozen lockfile, skip lifecycle scripts like prepare)
RUN pnpm install --frozen-lockfile --ignore-scripts

# Copy workspace package sources needed for frontend build
COPY packages/api-client/ packages/api-client/
COPY packages/branding/ packages/branding/
COPY packages/sdk/ packages/sdk/

# Build api-client first (frontend depends on its types)
RUN pnpm --filter @uwu/flora-api-client run build

# Copy frontend source and build
COPY apps/frontend/ apps/frontend/
RUN pnpm --filter frontend run build

# ---------------------------------------------------------------------------
# Stage 2: Build the Rust runtime (release)
# ---------------------------------------------------------------------------
FROM rust:1.94-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake clang libclang-dev pkg-config libssl-dev protobuf-compiler \
    python3 git \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /home/tasky/flora

# Copy full workspace source
COPY Cargo.toml Cargo.lock ./
COPY .cargo/ .cargo/
COPY apps/runtime/ apps/runtime/
COPY crates/ crates/
COPY packages/runtime-dist/ packages/runtime-dist/

# Embedded assets
COPY examples/basic/src/ examples/basic/src/
COPY --from=frontend /app/apps/frontend/dist apps/frontend/dist

# Fetch deps then build. Cache mounts keep Cargo registry/git state and compiled
# dependencies warm across GitHub Actions runs.
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git/db,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git/checkouts,sharing=locked \
    cargo fetch --locked

RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git/db,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git/checkouts,sharing=locked \
    --mount=type=cache,target=/tmp/flora-target,sharing=locked \
    CARGO_TARGET_DIR=/tmp/flora-target cargo build --release --locked --package flora \
    && cp /tmp/flora-target/release/flora /home/tasky/flora/flora

# ---------------------------------------------------------------------------
# Stage 3: Minimal runtime image
# ---------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
  && rm -rf /var/lib/apt/lists/*

RUN groupadd --gid 1000 flora \
  && useradd --uid 1000 --gid flora --create-home flora

COPY --from=builder /home/tasky/flora/flora /usr/local/bin/flora

USER flora
WORKDIR /home/flora

EXPOSE 3000

ENTRYPOINT ["flora"]
