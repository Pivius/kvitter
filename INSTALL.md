# INSTALLATION GUIDE

This guide will help you set up the Kvitter project backend and frontend on **Linux** and **Windows**.

## Prerequisites

- Rust (1.86+ recommended) - [Install Rust](https://rustup.rs/)
- Node.js and npm - [Install Node.js](https://nodejs.org/)
- PostgreSQL (version 16.9+ recommended)

## 1. Clone the repo

```bash
	git clone https://github.com/Pivius/kvitter.git
	cd kvitter
```

## 2. Backend Setup

### Common Environment Setup

Create a `.env` and a `.env.test` file inside `backend/`.
```env
	DATABASE_URL=postgres://user:password@localhost/kvitter_db
	JWT_SECRET=secret_key
```
Replace `secret_key`, `user` and `password` with your actual database user credentials.

For `.env.test`
```env
	DATABASE_URL=postgres://user:password@localhost/kvitter_test
	JWT_SECRET=secret_key
```

### Linux

**Install PostgreSQL**
```bash
	sudo apt install postgresql postgresql-contrib
```

**Create database user and database**
```bash
	sudo -u postgresql psql
```
Then inside psql:
```sql
	CREATE USER admin WITH PASSWORD 'password';
	CREATE DATABASE kvitter_db OWNER admin;
	CREATE DATABASE kvitter_test OWNER admin;
	GRANT ALL PRIVILEGES ON DATABASE kvitter_db TO admin;
	GRANT ALL PRIVILEGES ON DATABASE kvitter_test TO admin;
	\q
```

**Build the backend**
```bash
	cd backend
	cargo build --release
```

**Run database migrations**
Install `sqlx-cli`:
```bash
	cargo install sqlx-cli --no-default-features --features postgres,rustls
	sqlx migrate run
```

**Start the backend server**
```bash
	cargo run
```

### Windows

**Install PostgreSQL**
- Download and install [PostgreSQL](https://www.postgresql.org/download/windows/).
- During installation, remember your username and password

**Create database and user**
Open **SQL Shell (psql)**, then:
```sql
	CREATE USER admin WITH PASSWORD 'password';
	CREATE DATABASE kvitter_db OWNER admin;
	GRANT ALL PRIVILEGES ON DATABASE kvitter_db TO admin;
	\q
```

**Build the backend**
```powershell
	cargo build --release
```

**Install and run migrations**
Install `sqlx-cli`:
```powershell
	cargo install sqlx-cli --no-default-features --features postgres,rustls
	sqlx migrate run
```

**Start the backend server**
```powershell
	cargo run
```

## 3. Frontend setup (Vite + React)

```bash
	cd frontend
	npm install
	npm run dev
```

## 4. Troubleshooting

- Make sure `DATABASE_URL` in `.env` matches your PostgreSQL credentials
- if `psql` command is not found, make sure PostgreSQL `bin` folder is added to your system PATH.
- if ports are busy, stop other services using the same port or change ports in config.

## 5. Summary Scripts

You can run the provided scripts from the root folder:
- `./install.sh` - Installs dependencies and sets up environment.
- `./run.sh` - Runs backend and frontend concurrently.
- `./build.sh` - Builds backend and frontend.