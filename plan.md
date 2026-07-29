# Rust Restaurant System — Implementation Plan

## Overview

A restaurant ordering system where customers scan QR codes to order food, and staff (cashier, kitchen, waiter) manage orders via role-based dashboards. Built with Rust/Axum backend, Vue/Vite frontend, PostgreSQL, and Stripe for payments.

---

## Tech Stack

| Layer        | Technology                                     |
| ------------ | ---------------------------------------------- |
| Backend      | Rust, Axum                                     |
| Database     | PostgreSQL 18.1 (Docker)                       |
| ORM/Queries  | sqlx (compile-time checked queries)            |
| Migrations   | sqlx migrate                                   |
| Auth         | JWT (httpOnly cookie), argon2 password hashing  |
| Real-time    | Server-Sent Events (SSE)                       |
| QR Code      | `qrcode` crate (server-side SVG generation)    |
| Payments     | Stripe (Checkout Session / Payment Intents)    |
| Frontend     | Vue 3 + Vite + Vue Router + Pinia              |
| Styling      | Vanilla CSS (modern, premium design)           |
| Image Upload | Local filesystem (`uploads/`), served by Axum  |
| Language     | English only                                   |

---

## Project Structure

```
rust-restaurant-system/
├── Cargo.toml
├── Cargo.lock
├── .env                          # DB URL, JWT secret, Stripe keys, etc.
├── docker-compose.base.yml       # Postgres + Adminer (existing)
├── plan.md
├── uploads/                      # Menu item images (git-ignored)
├── migrations/                   # sqlx migrations
│   ├── 001_create_users.sql
│   ├── 002_create_tables.sql
│   ├── 003_create_categories.sql
│   ├── 004_create_menu_items.sql
│   ├── 005_create_orders.sql
│   ├── 006_create_order_items.sql
│   └── 007_seed_data.sql
├── src/
│   ├── main.rs                   # Entry point, Axum app setup
│   ├── config.rs                 # Env/config loading
│   ├── db.rs                     # Database pool setup
│   ├── error.rs                  # Unified error types
│   ├── models/                   # Database models (structs)
│   │   ├── mod.rs
│   │   ├── user.rs
│   │   ├── table.rs
│   │   ├── category.rs
│   │   ├── menu_item.rs
│   │   ├── order.rs
│   │   └── order_item.rs
│   ├── handlers/                 # Axum route handlers
│   │   ├── mod.rs
│   │   ├── auth.rs               # Login, logout, me
│   │   ├── user.rs               # Admin CRUD users
│   │   ├── table.rs              # CRUD tables, open/close, QR gen
│   │   ├── category.rs           # CRUD categories
│   │   ├── menu_item.rs          # CRUD menu items + image upload
│   │   ├── order.rs              # Create order (customer), list, checkout
│   │   ├── order_item.rs         # Update item status
│   │   ├── payment.rs            # Stripe checkout session, webhook
│   │   └── sse.rs                # SSE event stream
│   ├── middleware/
│   │   ├── mod.rs
│   │   └── auth.rs               # JWT extraction + role guard
│   ├── routes.rs                 # Route definitions
│   └── sse.rs                    # SSE broadcaster (tokio broadcast channel)
└── web/                          # Vue 3 + Vite frontend
    ├── package.json
    ├── vite.config.ts
    ├── index.html
    ├── public/
    ├── src/
    │   ├── main.ts
    │   ├── App.vue
    │   ├── router/
    │   │   └── index.ts          # Vue Router with role guards
    │   ├── stores/               # Pinia stores
    │   │   ├── auth.ts
    │   │   ├── menu.ts
    │   │   ├── order.ts
    │   │   └── table.ts
    │   ├── composables/
    │   │   ├── useApi.ts         # Axios/fetch wrapper
    │   │   └── useSse.ts         # SSE EventSource composable
    │   ├── views/
    │   │   ├── LoginView.vue
    │   │   ├── customer/
    │   │   │   └── OrderView.vue          # QR scanned → menu → cart → submit
    │   │   ├── cashier/
    │   │   │   ├── DashboardView.vue      # Table overview
    │   │   │   ├── TableDetailView.vue    # Open table, show QR, checkout
    │   │   │   └── MenuManageView.vue     # CRUD menu items
    │   │   ├── kitchen/
    │   │   │   ├── DashboardView.vue      # Live order queue
    │   │   │   └── MenuManageView.vue     # CRUD menu items
    │   │   ├── waiter/
    │   │   │   └── DashboardView.vue      # Order items to serve
    │   │   └── admin/
    │   │       ├── DashboardView.vue
    │   │       ├── UserManageView.vue     # CRUD users
    │   │       └── TableManageView.vue    # CRUD tables
    │   └── components/
    │       ├── MenuCard.vue
    │       ├── CartDrawer.vue
    │       ├── OrderItemCard.vue
    │       ├── QrCodeDisplay.vue
    │       ├── StatusBadge.vue
    │       └── AppLayout.vue
    └── ...
```

---

## Database Schema

### users

| Column         | Type         | Notes                              |
| -------------- | ------------ | ---------------------------------- |
| id             | UUID (PK)    | `gen_random_uuid()`                |
| username       | VARCHAR(100) | UNIQUE, NOT NULL                   |
| password_hash  | TEXT         | argon2 hashed                      |
| display_name   | VARCHAR(200) |                                    |
| role           | VARCHAR(20)  | `admin`, `cashier`, `kitchen`, `waiter` |
| is_active      | BOOLEAN      | default `true`                     |
| created_at     | TIMESTAMPTZ  | default `now()`                    |
| updated_at     | TIMESTAMPTZ  | default `now()`                    |

### tables

| Column         | Type         | Notes                              |
| -------------- | ------------ | ---------------------------------- |
| id             | UUID (PK)    |                                    |
| table_number   | INT          | UNIQUE, NOT NULL                   |
| name           | VARCHAR(100) | e.g., "Window Seat A"              |
| capacity       | INT          | optional                           |
| is_active      | BOOLEAN      | default `true`                     |
| created_at     | TIMESTAMPTZ  |                                    |
| updated_at     | TIMESTAMPTZ  |                                    |

### categories

| Column         | Type         | Notes                              |
| -------------- | ------------ | ---------------------------------- |
| id             | UUID (PK)    |                                    |
| name           | VARCHAR(100) | UNIQUE, NOT NULL                   |
| sort_order     | INT          | for display ordering               |
| is_active      | BOOLEAN      | default `true`                     |
| created_at     | TIMESTAMPTZ  |                                    |
| updated_at     | TIMESTAMPTZ  |                                    |

### menu_items

| Column         | Type           | Notes                            |
| -------------- | -------------- | -------------------------------- |
| id             | UUID (PK)      |                                  |
| category_id    | UUID (FK)      | → categories.id                  |
| name           | VARCHAR(200)   | NOT NULL                         |
| description    | TEXT           |                                  |
| price          | DECIMAL(10,2)  | NOT NULL                         |
| image_path     | TEXT           | relative path in `uploads/`      |
| is_available   | BOOLEAN        | default `true`                   |
| sort_order     | INT            |                                  |
| created_at     | TIMESTAMPTZ    |                                  |
| updated_at     | TIMESTAMPTZ    |                                  |

### orders

| Column         | Type           | Notes                                       |
| -------------- | -------------- | ------------------------------------------- |
| id             | UUID (PK)      |                                             |
| table_id       | UUID (FK)      | → tables.id                                 |
| session_token  | VARCHAR(64)    | UNIQUE, used in QR code URL                 |
| status         | VARCHAR(20)    | `open`, `checkout_pending`, `paid`, `closed` |
| total_amount   | DECIMAL(10,2)  | calculated on checkout                      |
| payment_method | VARCHAR(20)    | `stripe`, `cash`, `card`, `transfer`        |
| stripe_session_id | TEXT        | Stripe Checkout Session ID (nullable)       |
| opened_by      | UUID (FK)      | → users.id (cashier who opened)             |
| closed_by      | UUID (FK)      | → users.id (cashier who closed, nullable)   |
| opened_at      | TIMESTAMPTZ    | default `now()`                             |
| closed_at      | TIMESTAMPTZ    | nullable                                    |

### order_items

| Column         | Type           | Notes                                     |
| -------------- | -------------- | ----------------------------------------- |
| id             | UUID (PK)      |                                           |
| order_id       | UUID (FK)      | → orders.id                               |
| menu_item_id   | UUID (FK)      | → menu_items.id                           |
| quantity       | INT            | NOT NULL, default `1`                     |
| unit_price     | DECIMAL(10,2)  | snapshot of price at order time           |
| note           | TEXT           | customer special instructions             |
| status         | VARCHAR(20)    | `pending`, `preparing`, `finished`, `served` |
| created_at     | TIMESTAMPTZ    |                                           |
| updated_at     | TIMESTAMPTZ    |                                           |

---

## User Roles & Permissions

| Action                      | Admin | Cashier | Kitchen | Waiter |
| --------------------------- | ----- | ------- | ------- | ------ |
| Manage users                | ✅     | ❌       | ❌       | ❌      |
| Manage tables (CRUD)        | ✅     | ❌       | ❌       | ❌      |
| Open table / generate QR    | ✅     | ✅       | ❌       | ❌      |
| Checkout / accept payment   | ✅     | ✅       | ❌       | ❌      |
| Manage menu (CRUD)          | ✅     | ✅       | ✅       | ❌      |
| View orders                 | ✅     | ✅       | ✅       | ✅      |
| Update order item status    | ✅     | ❌       | ✅       | ✅      |

---

## API Routes

### Auth
```
POST   /api/auth/login          # Login → set JWT cookie
POST   /api/auth/logout         # Clear JWT cookie
GET    /api/auth/me             # Get current user info
```

### Users (admin only)
```
GET    /api/users               # List all users
POST   /api/users               # Create user
PUT    /api/users/:id           # Update user
DELETE /api/users/:id           # Deactivate user
```

### Tables (admin: CRUD, cashier: open/close)
```
GET    /api/tables              # List all tables
POST   /api/tables              # Create table (admin)
PUT    /api/tables/:id          # Update table (admin)
DELETE /api/tables/:id          # Deactivate table (admin)
POST   /api/tables/:id/open     # Open table → create order + session token + QR
POST   /api/tables/:id/close    # Close table (after payment)
GET    /api/tables/:id/qr       # Get QR code SVG for active session
```

### Categories
```
GET    /api/categories          # List categories
POST   /api/categories          # Create category
PUT    /api/categories/:id      # Update category
DELETE /api/categories/:id      # Deactivate category
```

### Menu Items
```
GET    /api/menu                # List menu items (with category)
POST   /api/menu                # Create menu item (multipart: data + image)
PUT    /api/menu/:id            # Update menu item
DELETE /api/menu/:id            # Deactivate menu item
POST   /api/menu/:id/image      # Upload/replace image
```

### Customer Ordering (public, token-based)
```
GET    /api/order/:token/menu   # Get available menu for this session
POST   /api/order/:token/items  # Submit order items
GET    /api/order/:token/status # Get order status and items
```

### Orders (staff)
```
GET    /api/orders              # List orders (filterable by status, table)
GET    /api/orders/:id          # Get order detail with items
```

### Order Items (kitchen/waiter)
```
PATCH  /api/order-items/:id/status  # Update item status
```

### Payments
```
POST   /api/orders/:id/checkout         # Initiate checkout (calculate total)
POST   /api/orders/:id/pay/stripe       # Create Stripe Checkout Session
POST   /api/orders/:id/pay/manual       # Record manual payment (cash/card/transfer)
POST   /api/webhooks/stripe             # Stripe webhook for payment confirmation
```

### SSE
```
GET    /api/events/kitchen      # SSE stream for kitchen (new orders, status changes)
GET    /api/events/waiter       # SSE stream for waiter (items ready to serve)
GET    /api/events/cashier      # SSE stream for cashier (payment updates, new orders)
```

### Static Files
```
GET    /uploads/*               # Serve uploaded images
```

---

## Key Flows

### 1. Cashier Opens a Table
1. Cashier logs in → JWT cookie set
2. Cashier clicks "Open" on a table
3. Backend creates an `order` with `status=open`, generates a unique `session_token`
4. Backend generates QR code SVG encoding URL: `{BASE_URL}/order/{session_token}`
5. QR is displayed on cashier screen (can be printed / shown to customer)

### 2. Customer Orders Food
1. Customer scans QR → opens `{BASE_URL}/order/{session_token}`
2. Vue app loads menu from `GET /api/order/{token}/menu`
3. Customer browses, adds items to cart, adds notes
4. Customer submits → `POST /api/order/{token}/items`
5. Backend creates `order_items` with `status=pending`
6. SSE broadcasts "new items" event to kitchen and cashier streams
7. Customer can add more items later (repeat steps 2-6)

### 3. Kitchen Processes Orders
1. Kitchen staff logs in → sees live order queue via SSE
2. Kitchen sees new order items appear in real-time
3. Kitchen clicks item → sets status to `preparing`
4. When done cooking → sets status to `finished`
5. SSE broadcasts status change to waiter stream

### 4. Waiter Serves Food
1. Waiter logs in → sees items with `status=finished`
2. Waiter delivers food → sets status to `served`
3. SSE broadcasts status change to kitchen and cashier

### 5. Checkout & Payment
1. Cashier initiates checkout → `POST /api/orders/:id/checkout` (calculates total)
2. **Stripe**: Cashier clicks "Pay with Stripe" → creates Stripe Checkout Session → customer/cashier completes payment → Stripe webhook confirms → order status = `paid`
3. **Manual**: Cashier records cash/card/transfer payment → order status = `paid`
4. Cashier closes table → order status = `closed`, session token invalidated

---

## SSE Event Types

```json
// New order items submitted by customer
{ "type": "new_order_items", "table_number": 5, "order_id": "uuid", "items": [...] }

// Order item status changed
{ "type": "item_status_changed", "table_number": 5, "order_id": "uuid", "item_id": "uuid", "status": "preparing" }

// Order checked out
{ "type": "order_checkout", "table_number": 5, "order_id": "uuid", "total": 450.00 }

// Payment received
{ "type": "payment_received", "table_number": 5, "order_id": "uuid", "method": "stripe" }
```

---

## Rust Backend Dependencies (Cargo.toml)

```toml
[dependencies]
# Web framework
axum = { version = "0.8", features = ["multipart"] }
tokio = { version = "1", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "fs"] }

# Database
sqlx = { version = "0.8", features = ["runtime-tokio", "tls-rustls", "postgres", "uuid", "chrono", "decimal"] }

# Auth
jsonwebtoken = "9"
argon2 = "0.5"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Utils
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
dotenvy = "0.15"
qrcode = "0.14"
image = "0.25"               # For QR code PNG rendering
rust_decimal = { version = "1", features = ["serde-with-str"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Stripe
async-stripe = "=v1.0.0-rc.6"
async-stripe-core = { version = "=v1.0.0-rc.6", features = ["customer"] }
```

---

## Vue Frontend Dependencies

```json
{
  "dependencies": {
    "vue": "^3.5",
    "vue-router": "^4.5",
    "pinia": "^3",
    "axios": "^1.9"
  },
  "devDependencies": {
    "vite": "^7",
    "@vitejs/plugin-vue": "^5",
    "typescript": "^5.8",
    "vue-tsc": "^2"
  }
}
```

---

## Seed Data

On first run (`007_seed_data.sql`):

- **Admin user**: `admin` / `admin` (role: admin)
- **Sample users**: `cashier1` / `password`, `kitchen1` / `password`, `waiter1` / `password`
- **Tables**: Table 1–10
- **Categories**: Appetizers, Main Course, Drinks, Desserts
- **Menu Items**: 3–4 items per category with placeholder descriptions and prices

---

## Implementation Phases

### Phase 1: Backend Core
1. Project setup: config, database pool, error handling
2. Database migrations (all tables)
3. Seed data migration
4. Auth: login/logout/me + JWT middleware + role guard
5. CRUD: users, tables, categories, menu items
6. Image upload for menu items
7. Customer ordering endpoints (token-based)
8. Order management + order item status updates
9. SSE broadcaster + event streams
10. QR code generation endpoint

### Phase 2: Frontend
1. Vue + Vite project setup in `web/`
2. Router setup with role-based guards
3. Login page
4. Customer ordering flow (scan → browse → cart → order → status)
5. Cashier POS dashboard (tables → open/close → QR → checkout)
6. Kitchen terminal (live order queue with SSE)
7. Waiter view (items to serve with SSE)
8. Menu management views (shared by cashier/kitchen)
9. Admin views (user management, table management)

### Phase 3: Stripe Integration
1. Stripe Checkout Session creation
2. Stripe webhook handler
3. Payment status sync
4. Frontend payment flow in cashier checkout

### Phase 4: Polish
1. Error handling & validation
2. Responsive design for tablet/mobile (kitchen/waiter often use tablets)
3. Print-friendly QR code view
4. Loading states, toast notifications
5. Testing

---

## Environment Variables (.env)

```env
DATABASE_URL=postgres://postgres:example@localhost:5432/restaurant
JWT_SECRET=your-secret-key-change-in-production
JWT_EXPIRY_HOURS=24
BASE_URL=http://localhost:3000
UPLOAD_DIR=./uploads
STRIPE_SECRET_KEY=sk_test_...
STRIPE_WEBHOOK_SECRET=whsec_...
RUST_LOG=info
```
