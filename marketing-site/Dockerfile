FROM node:18-bullseye AS builder
WORKDIR /app
COPY marketing-site/package.json marketing-site/package-lock.json* ./
RUN npm ci || npm install
COPY marketing-site .
RUN npm run build

FROM node:18-bullseye
WORKDIR /app
ENV NODE_ENV=production
COPY --from=builder /app .
EXPOSE 4000
CMD ["npm","run","start"]

