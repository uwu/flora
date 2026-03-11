FROM node:20-slim AS base

WORKDIR /app
ENV NODE_ENV=production

FROM base AS build
RUN corepack enable
COPY pnpm-workspace.yaml ./
COPY tsconfig.json /tsconfig.json
COPY apps/build-service/package.json apps/build-service/pnpm-lock.yaml ./
COPY apps/build-service/tsconfig.json apps/build-service/tsdown.config.ts ./
RUN pnpm install --frozen-lockfile
COPY apps/build-service/src ./src
RUN pnpm run build

FROM base AS runtime
RUN corepack enable
COPY pnpm-workspace.yaml ./
COPY apps/build-service/package.json apps/build-service/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile --prod
COPY --from=build /app/dist ./dist
RUN mkdir -p /app/build-workspace \
  && chown -R node:node /app
USER node
EXPOSE 3001
CMD ["node", "dist/index.mjs"]
