# Jakewray.dev Project Context

## Project Overview

This repository contains the source code for [jakewray.dev](https://jakewray.dev), a personal portfolio website.
The application is a modern, full-stack Rust web application utilizing Server-Side Rendering (SSR).

### Core Technologies

- **Frontend**: Leptos (Rust WASM framework)
- **Backend**: Axum (Rust async web framework)
- **Database**: SQLite (managed via SQLx)
- **Styling**: SASS / SCSS
- **Environment**: Nix (via `flake.nix` and `direnv`)
- **Deployment**: Managed externally via a meta repo.

## Directory Structure

- `backend/`: Server-side Rust code. Houses the Axum server, API handlers, database connection pools, and Leptos app serving setup.
- `frontend/`: Client-side Rust code. Contains Leptos components, routing, and UI logic.
- `shared/`: Shared types, models, and utilities used by both frontend and backend.
- `migrations/` & `migration/`: SQLx database migration files.
- `scripts/`: Automation scripts for local development database setup.
- `style/`: SASS stylesheets.
- `.github/workflows/`: CI/CD pipelines (Formatting, Linting, Testing, Security Audits, and AI reviews).

## Workflow & Development

### Local Environment

The project heavily utilizes Nix for reproducible development environments.

1. Allow the environment: `direnv allow`
2. Initialize the dev database: `./scripts/setup-dev.sh`

### Running the Application

The recommended way to run the application in development is via `cargo-leptos`, which provides hot-reloading:

```bash
cargo leptos watch
```

*Note: Make sure your local SQLite database is initialized via `./scripts/setup-dev.sh`.*

## Important Architectural Guidelines for AI Assistants

1. **Leptos SSR Boundaries**:
   Because this is an SSR application, code is compiled for both the server (`wasm32-unknown-unknown` target is NOT used for SSR, standard target is) and the client (`wasm32-unknown-unknown`).
   - Use `#[cfg(feature = "ssr")]` to gate server-side-only logic (like database access with `sqlx`, file system operations, etc.).
   - Use `ServerFn` for RPC communication between the client components and the Axum backend.

2. **SQLx Compile-Time Checks**:
   Database queries are verified at compile time by SQLx. When adding or changing queries, ensure `DATABASE_URL` is correctly set in the environment, or run `cargo sqlx prepare` to update the `.sqlx` offline data before committing.

3. **Admin and Authentication**:
   The project has an admin panel that uses Argon2 for password hashing and JWT for session management. When dealing with authenticated routes in the `frontend`, ensure proper signal checks and server function authentication state logic.

4. **Styling**:
   Global styling is handled via SASS. When adding new components, add corresponding styles to the `style/` directory and ensure they are compiled correctly by the Leptos build pipeline.


- Tick off tasks in the roadmap as they are completed.
- Update the roadmap as the project progresses.
- Update the plan as the project progresses.
- Update the GEMINI.md SPARINGLY as the project progresses.
- Update the README.md as the project progresses.
- run cargo nextest, cargo fmt, and cargo clippy after every task.
