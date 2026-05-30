FROM oven/bun:1.3.14 AS build
WORKDIR /app

ARG VITE_API_URL=http://localhost:3000
ENV VITE_API_URL=${VITE_API_URL}

COPY package.json bun.lock bunfig.toml tsconfig.json tsconfig.base.json ./
COPY apps/api/package.json apps/api/package.json
COPY apps/ui/package.json apps/ui/package.json
COPY packages/contracts/package.json packages/contracts/package.json
RUN bun install --frozen-lockfile

COPY . .
RUN bun run --filter @bambu-monitor/ui build

FROM nginx:1.27-alpine AS runtime

COPY docker/nginx.ui.conf /etc/nginx/conf.d/default.conf
COPY --from=build /app/apps/ui/dist /usr/share/nginx/html

EXPOSE 80
