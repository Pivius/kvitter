# Table of Contents

- [Table of Contents](#table-of-contents)
- [Architecture Overview](#architecture-overview)
- [Components](#components)
  - [Frontend](#frontend)
  - [Backend](#backend)
- [Database](#database)
- [Authentication](#authentication)
- [Deployment](#deployment)
- [Testing](#testing)
- [Development Workflow](#development-workflow)

# Architecture Overview

Kvitter is a full-stack web application designed for personal productivity and finance tracking, 
especially for freelancers and independent workers. 
The system is split into a Rust-powered backend and a React-based frontend, 
communicating via RESTful APIs. PostgreSQL is used as the primary database.

# Components

## Frontend

- **Framework:** React (with Vite for fast development and build)
- **Language:** TypeScript
- **Styling:** Tailwind CSS (or your chosen CSS framework)
- **State Management:** React Context or Redux (if used)
- **API Communication:** Uses `fetch` or `axios` to interact with backend REST APIs
- **Features:**
  - User authentication (login/signup)
  - Dashboard for income/expense tracking
  - Data visualization (charts, tables)
  - Responsive design for desktop and mobile
- **Location:** [`frontend/`](frontend/)
- 
## Backend

- **Framework:** [Axum](https://github.com/tokio-rs/axum) (Rust web framework)
- **Language:** Rust
- **Database ORM:** [SQLx](https://github.com/launchbadge/sqlx) (async, compile-time checked queries)
- **Authentication:** JWT-based authentication
- **Features:**
  - RESTful API endpoints for user management, finance data, etc.
  - Secure password hashing (e.g., Argon2 or bcrypt)
  - Input validation and error handling
  - Modular route organization
- **Location:** [`backend/`](backend/)

# Database

- **Type:** PostgreSQL
- **Schema:**
  - `users` table: stores user credentials and profile info
  - `transactions` table: stores income and expense records
  - (Add more tables as needed for categories, tags, etc.)
- **Migrations:** Managed with `sqlx-cli` and stored in [`backend/migrations/`](backend/migrations/)
- **Connection:** Configured via the `DATABASE_URL` environment variable

# Authentication

- **Method:** JWT (JSON Web Token)
- **Flow:**
  1. User signs up or logs in with email and password.
  2. Backend verifies credentials and returns a JWT on success.
  3. JWT is stored client-side (e.g., in localStorage) and sent in the `Authorization: Bearer <token>` header for protected API requests.
  4. Backend validates JWT on each request to protected endpoints.
- **Security:**
  - Passwords are hashed before storage.
  - JWT secret is stored in environment variables, never in source code.
  - Token expiration is enforced (e.g., 24 hours).
- **Hashing:**
  - Uses Argon2id for secure password hashing, with a unique salt for each user.

# Deployment

- **Backend:**
  - Build with `cargo build --release`
  - Run as a systemd service, Docker container, or on a cloud platform (e.g., Heroku, AWS, DigitalOcean)
  - Environment variables set via `.env` or deployment secrets
- **Frontend:**
  - Build with `npm run build` (output in `frontend/dist`)
  - Serve with a static file server (e.g., Nginx, Vercel, Netlify)
- **Database:**
  - Hosted PostgreSQL instance (managed or self-hosted)
  - Migrations run via `sqlx migrate run`

# Testing

- **Backend:**
  - Unit and integration tests using Rust’s built-in test framework and `sqlx::test`
  - Test database configured via `.env.test`
  - Example: `cargo test` in the `backend/` directory
- **Frontend:**
  - Unit and component tests using Jest, React Testing Library, or Vitest
  - Example: `npm test` or `npm run test` in the `frontend/` directory
- **End-to-End:**
  - (Optional) Use Cypress or Playwright for full-stack E2E tests

# Development Workflow

1. **Clone the repository:**  
   `git clone https://github.com/Pivius/kvitter.git && cd kvitter`

2. **Install dependencies:**  
   - Backend: `cd backend && cargo build`
   - Frontend: `cd frontend && npm install`

3. **Set up environment variables:**  
   - Copy `.env.example` to `.env` in `backend/` and fill in your secrets.

4. **Run database migrations:**  
   - `cd backend && sqlx migrate run`

5. **Start development servers:**  
   - Use `./run.sh` to start both backend and frontend concurrently.

6. **Build for production:**  
   - Use `./build.sh` to build both backend and frontend.

7. **Testing:**  
   - Backend: `cargo test`
   - Frontend: `npm run test`

8. **Contributing:**  
   - Follow code style guidelines.
   - Write tests for new features.
   - Document your code and update this architecture file as needed.

---

For more details on installation and setup, see [INSTALL.md](INSTALL.md).