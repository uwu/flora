FROM node:22-slim AS base

WORKDIR /app
ENV NODE_ENV=production
RUN corepack enable

FROM base AS build
COPY package.json pnpm-workspace.yaml pnpm-lock.yaml ./
COPY apps/build-service/package.json apps/build-service/
COPY apps/build-service/tsconfig.json apps/build-service/vite.config.ts apps/build-service/
RUN pnpm install --frozen-lockfile --ignore-scripts --filter @flora-internal/build-service...
COPY apps/build-service/src apps/build-service/src
WORKDIR /app/apps/build-service
RUN pnpm run build

FROM base AS runtime
COPY package.json pnpm-workspace.yaml pnpm-lock.yaml ./
COPY apps/build-service/package.json apps/build-service/
RUN pnpm install --frozen-lockfile --prod --ignore-scripts --filter @flora-internal/build-service...
COPY --from=build /app/apps/build-service/dist /app/apps/build-service/dist
WORKDIR /app/apps/build-service
RUN mkdir -p /app/build-workspace \
  && chown -R node:node /app
USER node
EXPOSE 3001
CMD ["node", "dist/index.mjs"]
