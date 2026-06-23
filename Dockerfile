FROM node:20-alpine AS web-build
WORKDIR /usr/src/app
COPY frontend/package.json frontend/package-lock.json* ./
RUN npm install
COPY frontend ./frontend
RUN npm run build --prefix frontend

FROM rust:1.80-slim AS backend-build
WORKDIR /usr/src/app
COPY backend/Cargo.toml backend/Cargo.toml
COPY backend/src ./backend/src
RUN mkdir -p /usr/src/app/target
RUN cargo build --manifest-path backend/Cargo.toml --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=backend-build /usr/src/app/target/release/etcdpilot-backend /app/etcdpilot
COPY --from=web-build /usr/src/app/frontend/dist /app/web/dist
COPY config.toml /etc/etcdpilot/config.toml
RUN mkdir -p /var/lib/etcdpilot /app/web/dist
ENV ETCD_MANAGER_CONFIG=/etc/etcdpilot/config.toml
EXPOSE 8080
CMD ["/app/etcdpilot"]
