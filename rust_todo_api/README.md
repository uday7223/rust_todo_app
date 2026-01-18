# Rust Todo API (Axum + SQLx + JWT)

This project is a beginner-friendly Rust backend with:

- Axum for HTTP routing
- SQLx for PostgreSQL access (async)
- JWT auth with jsonwebtoken
- Password hashing with Argon2

The goal is to show a clean, minimal flow from register -> login -> protected
todos.

## Project Structure

```
src/
  main.rs    // app startup + router wiring
  db.rs      // database connection pool
  auth.rs    // password hashing + JWT + auth middleware
  models.rs  // request/response DTOs
  routes.rs  // HTTP handlers
```

## Setup

1) Create database and tables

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE todos (
    id UUID PRIMARY KEY,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    completed BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT NOW()
);
```

2) Create `.env`

```
DATABASE_URL=postgres://<user>:<password>@localhost:5432/rust_todo
JWT_SECRET=supersecretlongstringhere
```

## Run

```
DATABASE_URL=postgres://<user>:<password>@localhost:5432/rust_todo cargo run
```

If you exported `DATABASE_URL` already, you can just run `cargo run`.

## High-Level Flow

1) Register:
   - Client sends email + password.
   - Server hashes the password and inserts the user.

2) Login:
   - Server fetches the user by email.
   - Verifies the password hash.
   - Creates a JWT with `sub = user_id` and a 1-day expiry.

3) Todos:
   - Client includes `Authorization: Bearer <token>`.
   - Middleware verifies token and inserts `Uuid` into request extensions.
   - Handlers read `Extension<Uuid>` as the logged-in user id.

## Important Files and Responsibilities

### `src/main.rs`
- Loads `.env`
- Creates the DB pool
- Wires public and protected routes

Protected routes are nested under `/todos` and wrapped in the JWT middleware.

### `src/db.rs`
- Defines `connect_db()` which builds a SQLx pool.

### `src/auth.rs`
- `hash_password()` and `verify_password()` using Argon2
- `generate_jwt()` creates tokens with a 1-day expiry
- `auth_middleware()` validates Bearer tokens and injects `Uuid` into extensions

### `src/models.rs`
- Defines request/response structs:
  - `RegisterReq`
  - `LoginReq`
  - `CreateTodoReq`
  - `TodoResponse`

### `src/routes.rs`
Handlers for:
- `POST /register`
- `POST /login`
- `POST /todos` (protected)
- `GET /todos` (protected)

## Endpoints

### Register

```
curl -X POST http://localhost:3002/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"secret123"}'
```

### Login

```
curl -X POST http://localhost:3002/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"secret123"}'
```

Response:

```
{"token":"<jwt>"}
```

### Create Todo (protected)

```
curl -X POST http://localhost:3002/todos \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <jwt>" \
  -d '{"title":"Learn Rust"}'
```

### List Todos (protected)

```
curl -X GET http://localhost:3002/todos \
  -H "Authorization: Bearer <jwt>"
```

## Common Errors

### "role does not exist"
Your `DATABASE_URL` user does not match a Postgres role on your machine. Check
pgAdmin "Login/Group Roles" or run `psql -d postgres -c "\\du"`.

### "Missing request extension: Extension<Uuid>"
The JWT middleware did not run, or the route is not protected. Ensure `/todos`
is nested under a router that uses `auth_middleware`.

### Compile-time SQLx errors
This project uses `sqlx::query` (runtime validation), so compile-time DB access
is not required.

## Next Steps (Optional)

- Add update and delete todo endpoints
- Add input validation (email format, password length)
- Add refresh tokens
- Use SQLx migrations and `sqlx::migrate!`
