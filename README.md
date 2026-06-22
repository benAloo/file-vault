# File Vault Monorepo

File Vault is a modern, production-grade file upload and storage platform designed for high performance, cost efficiency, and security. The project is organized as a workspace monorepo containing a Rust-based metadata API, a React frontend utilizing Tailwind CSS v4, and integrated Supabase services.

---

## 🚀 Tech Stack

- **Frontend:** React 19, TypeScript, Vite, Tailwind CSS v4, TanStack Query
- **Backend:** Rust, Axum, SQLx
- **Infrastructure & Storage:** Supabase (PostgreSQL, Supabase Auth, Supabase Storage)
- **Monorepo Orchestration:** PNPM Workspaces (Node) & Cargo Workspaces (Rust)
- **Security & Compliance:** ClamAV Malware Scanning, Magic Bytes Verification, SHA-256 Deduplication, and PostgreSQL Row-Level Security (RLS)

---

## 📁 Repository Structure

```text
file-vault/
├── apps/
│   ├── api/                     # Rust backend (Axum)
│   └── web/                     # React frontend (Vite + Tailwind v4)
├── packages/
│   └── types/                   # Shared TypeScript/Rust type definitions
├── supabase/
│   ├── migrations/              # Local database migrations & schemas
│   └── config.toml              # Supabase local engine configuration
├── Cargo.toml                   # Root Cargo workspace manifest
├── pnpm-workspace.yaml          # Root PNPM workspace manifest
└── AGENTS.md                    # Detailed coding rules & CI gates for AI-agents
```

---

## 🛠️ Local Prerequisites

Before setting up the project, verify you have the following installed on your local machine:

1. **Node.js** (v20.0.0 or higher)
2. **PNPM** (v9.0.0 or higher)
3. **Rust & Cargo** (Stable toolchain)
4. **Supabase CLI** (For running local migrations and storage buckets)
5. **Docker Desktop** (Required by the Supabase CLI to emulate database/storage locally)

---

## 🏁 Quick Start & Local Development

Follow these steps to spin up the local development environment:

### 1. Install Workspace Dependencies

From the repository root, install dependencies for all package workspaces:

```bash
pnpm install
```

### 2. Start the Local Supabase Engine

Ensure your Docker daemon is running, then initialize and start the local Supabase services:

```bash
# Start local Postgres, Auth, and Storage emulators
supabase start
```

This command spins up the backend database and prints your local API keys, project URL, and database connection strings to your terminal.

### 3. Spin Up the Services

Open two separate terminal sessions to run the backend and frontend simultaneously:

#### Run the Rust Backend (API)

Navigate to the API app or use Cargo to run the target directly:

```bash
cargo run --bin api
```

#### Run the React Frontend (Web)

Use PNPM to run the Vite dev server for the web app workspace:

```bash
pnpm --filter web dev
```

---

## 📋 Workspace Commands Cheat Sheet

Run workspace actions from the repository root using these commands:

| Action | Command | Scope |
| :--- | :--- | :--- |
| **Install** | `pnpm install` | Global |
| **Run Web Dev** | `pnpm --filter web dev` | Frontend |
| **Build Web** | `pnpm --filter web build` | Frontend |
| **Lint Web** | `pnpm --filter web lint` | Frontend |
| **Run Rust API** | `cargo run --bin api` | Backend |
| **Check Rust** | `cargo clippy --all-targets` | Backend |
| **Stop Database** | `supabase stop` | Database |

---

## 🛡️ Coding Standards & Contribution Guardrails

We enforce strict security, architecture, and quality gates across both the Rust and React workspaces. If you are using AI-assisted programming tools (such as Cursor, Copilot, or custom code generation agents), refer to the [AGENTS.md](./AGENTS.md) file in the root of the repository. It contains strict configuration rules, styling guidelines for Tailwind CSS v4, security verification requirements (Magic Bytes, RLS), and our pipeline's **Definition of Done (DoD)**.
