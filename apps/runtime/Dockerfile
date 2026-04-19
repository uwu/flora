# ---------------------------------------------------------------------------
# Stage 1: Build the frontend
# ---------------------------------------------------------------------------
FROM node:22-slim AS frontend

WORKDIR /app
RUN corepack enable

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

# Use the same absolute path as the source tree so that file:// git deps in
# Cargo.toml (e.g. submodules/deno_core) resolve correctly.
WORKDIR /home/tasky/flora

# Disable sccache wrapper (set in .cargo/config.toml but not available in Docker)
ENV RUSTC_WRAPPER=""

# Copy full workspace source
COPY Cargo.toml Cargo.lock ./
COPY .cargo/ .cargo/
COPY apps/runtime/ apps/runtime/
COPY crates/ crates/
COPY packages/runtime-dist/ packages/runtime-dist/
COPY submodules/ submodules/

# Embedded assets
COPY examples/basic/src/ examples/basic/src/
COPY --from=frontend /app/apps/frontend/dist apps/frontend/dist

# Fetch deps then build
RUN cargo fetch --locked
RUN cargo build --release --package flora

# ---------------------------------------------------------------------------
# Stage 3: Minimal runtime image
# ---------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
  && rm -rf /var/lib/apt/lists/*

RUN groupadd --gid 1000 flora \
  && useradd --uid 1000 --gid flora --create-home flora

COPY --from=builder /home/tasky/flora/target/release/flora /usr/local/bin/flora

USER flora
WORKDIR /home/flora

EXPOSE 3000

ENTRYPOINT ["flora"]
