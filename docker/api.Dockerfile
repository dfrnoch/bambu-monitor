ARG BUN_VERSION=1.3.14

FROM oven/bun:${BUN_VERSION} AS build
WORKDIR /app

COPY package.json bun.lock bunfig.toml tsconfig.json tsconfig.base.json ./
COPY apps/api/package.json apps/api/package.json
COPY apps/ui/package.json apps/ui/package.json
COPY packages/contracts/package.json packages/contracts/package.json
RUN bun install --frozen-lockfile

COPY . .
RUN bun run --filter @bambu-monitor/api build

FROM oven/bun:${BUN_VERSION} AS runtime
WORKDIR /app/apps/api

ENV NODE_ENV=production
ENV PORT=3000
ENV DATA_DIR=/data
ENV AUTO_CONNECT=true

COPY --from=build /app/apps/api/dist ./dist

EXPOSE 3000
VOLUME ["/data"]

CMD ["bun", "dist/index.js"]
