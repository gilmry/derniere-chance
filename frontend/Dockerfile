# Multi-stage build: compile the static Astro/Svelte PWA, serve it with nginx.
# Built by CI (.github/workflows/docker-publish.yml), not on the target VPS.

FROM node:22-slim AS builder
WORKDIR /app

ARG PUBLIC_API_URL=https://api.derniere-chance.ecosolva.org
ENV PUBLIC_API_URL=${PUBLIC_API_URL}

COPY package.json package-lock.json* ./
RUN npm install

COPY . .
RUN npm run build

# Pre-compress once at build time - nginx serves the .gz sibling directly
# (gzip_static), avoiding re-gzipping on every request on a shared 1-vCPU box.
RUN find dist -type f \( -name '*.js' -o -name '*.css' -o -name '*.html' \
      -o -name '*.svg' -o -name '*.json' \) -exec gzip -9 -k {} \;

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80
