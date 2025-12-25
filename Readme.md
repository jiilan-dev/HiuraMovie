# HiuraMovie Backend 🎬

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![Axum](https://img.shields.io/badge/axum-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![Postgres](https://img.shields.io/badge/postgres-%23316192.svg?style=for-the-badge&logo=postgresql&logoColor=white)
![Docker](https://img.shields.io/badge/docker-%230db7ed.svg?style=for-the-badge&logo=docker&logoColor=white)

High-performance streaming backend for HiuraMovie, built with **Rust** and **Axum**.  
Designed to handle video streaming, high-concurrency requests, and complex background jobs.

🔗 **Repository**: [https://github.com/jiilan-dev/HiuraMovie](https://github.com/jiilan-dev/HiuraMovie)

---

## 🏗 Tech Stack

- **Core**: Rust (2024 Edition)
- **Web Framework**: [`axum`](https://github.com/tokio-rs/axum) (Ergonomic & Modular)
- **Database**: PostgreSQL 16 (via [`sqlx`](https://github.com/launchbadge/sqlx))
- **Caching**: Redis 7
- **Object Storage**: AWS S3 / MinIO (for video/image storage)
- **Message Queue**: RabbitMQ (via `lapin`)
- **Authentication**: JWT + Argon2 Hashing
- **Observability**: `tracing`

## 🧩 Architecture

The project follows a **Feature-based Clean Architecture**:

\`\`\`
src/
├── modules/           # Domain features (Auth, User, Subscription, etc.)
│   ├── auth/          # Each module contains:
│   │   ├── handler.rs # HTTP Interface
│   │   ├── service.rs # Business Logic
│   │   └── model.rs   # Domain Models
├── infrastructure/    # External services (DB, S3, Redis access)
├── common/            # Shared utilities
└── config/            # Environment configurations
\`\`\`

## 🚀 Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Docker](https://docs.docker.com/get-docker/) & Docker Compose
- `sqlx-cli`: `cargo install sqlx-cli`

### 1. Setup Environment

\`\`\`bash
# Copy the example env file
cp .env.example .env
\`\`\`

### 2. Start Infrastructure

Start Postgres, Redis, and MinIO containers:

\`\`\`bash
docker compose up -d
\`\`\`

### 3. Run Migrations

Initialize the database schema:

\`\`\`bash
# Ensure the database is ready
bash scripts/migrate.sh
\`\`\`

### 4. Run the Server

Start the development server with hot-reload (requires `cargo-watch`):

\`\`\`bash
bash scripts/dev.sh
# OR
cargo run
\`\`\`

Server will be running at `http://localhost:3000`

## 🛠 Features (In Progress)

- [x] **Authentication**: Secure Login/Register with JWT.
- [ ] **Video Streaming**: HTTP Range requests for seeking support (`axum-range`).
- [ ] **Transcoding**: Background jobs to convert uploads to HLS/DASH.
- [ ] **Watch History**: Tracking playback progress with Redis.

## 🤝 Contributing

1. Fork the repo.
2. Create your feature branch (`git checkout -b feature/amazing-feature`).
3. Commit your changes.
4. Push to the branch.
5. Open a Pull Request.
