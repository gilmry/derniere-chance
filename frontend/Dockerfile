# Multi-stage build: compile the static Astro/Svelte PWA, serve it with nginx.
# Built by CI (.github/workflows/docker-publish.yml), not on the target VPS.

FROM node:22-slim AS builder
WORKDIR /app

ARG PUBLIC_API_URL=https://api.derniere-chance.ecosolva.org
ENV PUBLIC_API_URL=${PUBLIC_API_URL}

ARG PUBLIC_DOMAIN=derniere-chance.ecosolva.org
ENV PUBLIC_DOMAIN=${PUBLIC_DOMAIN}
ARG PUBLIC_LEGAL_EDITOR_NAME="Gilles Maury"
ENV PUBLIC_LEGAL_EDITOR_NAME=${PUBLIC_LEGAL_EDITOR_NAME}
ARG PUBLIC_LEGAL_EDITOR_STATUS="indépendant (architecte solutions & développeur freelance)"
ENV PUBLIC_LEGAL_EDITOR_STATUS=${PUBLIC_LEGAL_EDITOR_STATUS}
ARG PUBLIC_LEGAL_EDITOR_CITY="Bruxelles, Belgique"
ENV PUBLIC_LEGAL_EDITOR_CITY=${PUBLIC_LEGAL_EDITOR_CITY}
ARG PUBLIC_LEGAL_BUSINESS_NUMBER=BE0670.778.061
ENV PUBLIC_LEGAL_BUSINESS_NUMBER=${PUBLIC_LEGAL_BUSINESS_NUMBER}
ARG PUBLIC_LEGAL_CONTACT_EMAIL=gilmry@gmail.com
ENV PUBLIC_LEGAL_CONTACT_EMAIL=${PUBLIC_LEGAL_CONTACT_EMAIL}
ARG PUBLIC_GITHUB_REPO=gilmry/derniere-chance
ENV PUBLIC_GITHUB_REPO=${PUBLIC_GITHUB_REPO}

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
