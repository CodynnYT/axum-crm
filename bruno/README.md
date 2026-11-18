# Axum CRM API collection

Open this directory in Bruno and select the Local environment. Start the backend with PostgreSQL, `DATABASE_URL` and `JWT_SECRET` configured. The default port is 3000. Update `baseUrl` if needed.

Set `email` and `password` to an existing account, run Auth/Login, and use the captured token for authenticated requests. The example credentials match the backend's default first-admin configuration only when that account was created with those defaults. Do not use real credentials in a shared environment file. If token capture is unavailable, copy `token` from the login response into the environment manually.

Replace the example IDs with IDs returned by create or list requests. The `customer_id` in deal examples must reference an existing customer; set it to `null` if no customer is available. The list requests have disabled example search filters that can be enabled in Bruno. User creation and deletion require an admin token; deleting the signed-in user is prohibited.

Create, update, delete, convert, complete and password-change requests modify persistent data. Lead conversion deletes the lead and creates a customer. Password change alters the account credentials. Use a disposable database and review each request before sending. The collection is not intended to be run sequentially as an automated destructive suite.

The collection documents all registered API method/path combinations, not static file serving or framework-generated responses. Examples cover typical successful inputs, not every validation error or optional field combination. A running backend and database are required to exercise the requests.