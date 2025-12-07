FROM node:18-bullseye AS builder
WORKDIR /app
COPY docs-portal/package.json docs-portal/package-lock.json* ./
RUN npm ci || npm install
COPY docs-portal .
RUN npm run build

FROM nginx:1.27-alpine
COPY --from=builder /app/build /usr/share/nginx/html
EXPOSE 80
CMD ["nginx","-g","daemon off;"]

