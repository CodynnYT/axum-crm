# Axum CRM

A Rust CRM backend built with Axum, SQLx, and PostgreSQL. It provides JSON APIs for leads, customers, deals, activities, reports, and users. No frontend is included in this repository.

## Setup

Requirements: a Rust toolchain supporting edition 2024, Cargo, and a running PostgreSQL server. Create an empty database and provide a connection URL for a user with permission to create tables and run migrations.

From the repository root, configure the environment and run the backend. For example, in PowerShell:

```powershell
cd backend
$env:DATABASE_URL = 'postgres://postgres:password@localhost:5432/crm'
$env:JWT_SECRET = 'replace-with-a-long-random-secret'
$env:ADMIN_EMAIL = 'admin@example.com'
$env:ADMIN_PASSWORD = 'replace-with-a-strong-password'
cargo run
```

The backend also loads a `.env` file through `dotenvy`. Startup connects to PostgreSQL, runs the SQLx migrations, creates an initial admin if there are no users, and listens on `0.0.0.0:3000` by default. The migrations insert demo leads, customers, deals, and activities into a fresh database.

### Configuration

| Variable | Description | Default |
| --- | --- | --- |
| `DATABASE_URL` | PostgreSQL connection URL | Required |
| `JWT_SECRET` | Secret used to sign tokens | Required |
| `PORT` | HTTP port | `3000` |
| `ADMIN_NAME` | Initial admin display name | `Admin` |
| `ADMIN_EMAIL` | Initial admin email | `admin@crm.local` |
| `ADMIN_PASSWORD` | Initial admin password | `admin12345` |
| `STATIC_DIR` | Optional directory intended for static files | Unset |
| `RUST_LOG` | Tracing filter | `info` |

Set a strong admin password and JWT secret before first startup. Admin settings only apply when the users table is empty; changing them later does not update an existing account. The default password is unsuitable for deployment. The router supports static files with an `index.html` fallback when `STATIC_DIR` is set. CORS is permissive in the current router.

## API

All routes are prefixed with `/api`. Health and login are public; the other routes require `Authorization: Bearer <token>`. Login accepts JSON containing `email` and `password`, and returns a token and user. Tokens expire after 24 hours. Passwords are hashed with Argon2. User creation and deletion require the `admin` role.

| Method | Path | Purpose |
| --- | --- | --- |
| GET | `/api/health` | Check database connectivity |
| POST | `/api/auth/login` | Sign in |
| GET, PUT | `/api/auth/me` | View or update the current user |
| PUT | `/api/auth/password` | Change the current user's password |
| GET | `/api/dashboard` | Dashboard summary |
| GET, POST | `/api/leads` | List or create leads |
| GET, PUT, DELETE | `/api/leads/{id}` | View, update, or delete a lead |
| POST | `/api/leads/{id}/convert` | Convert a lead to a customer |
| GET, POST | `/api/customers` | List or create customers |
| GET, PUT, DELETE | `/api/customers/{id}` | View, update, or delete a customer |
| GET, POST | `/api/deals` | List or create deals |
| GET, PUT, DELETE | `/api/deals/{id}` | View, update, or delete a deal |
| GET, POST | `/api/activities` | List or create activities |
| GET, PUT, DELETE | `/api/activities/{id}` | View, update, or delete an activity |
| POST | `/api/activities/{id}/complete` | Complete an activity |
| GET, POST | `/api/reports` | List or create reports |
| GET, DELETE | `/api/reports/{id}` | View a report summary or delete a report |
| GET | `/api/reports/{id}/csv` | Download report deals as CSV |
| GET, POST | `/api/users` | List users or create a user |
| DELETE | `/api/users/{id}` | Delete a user |

## Bruno collection

Open the `bruno` folder in Bruno, select the `Local` environment, and run the login request before authenticated requests. See [the collection instructions](bruno/README.md) for credentials, IDs, and destructive requests.

## Tests

Run the backend tests from `backend` with `cargo test`.