# 🍽️ Rustaurant — Rust Restaurant System

A full-stack, real-time restaurant management system built with high-performance Rust (Axum, SQLx, Tokio) and Vue 3 (TypeScript, Vite). Features QR code customer ordering, real-time Kitchen & Waiter displays powered by Server-Sent Events (SSE), Cashier POS, and comprehensive Admin management.

---

## 📸 Demo Showcase

#### Table Overview
![Table View](screenshots/table-view.png)

#### Table Detail & POS
![Table Detail View](screenshots/table-detail-view.png)

#### Customer Order View
![Customer View](screenshots/customer-view.png)

#### Kitchen Dashboard
![Kitchen View](screenshots/kitchen-view.png)

#### Orders Overview
![Order View](screenshots/order-view.png)

#### Food Menu & Category Management
![Food Menu Manage View](screenshots/food-menu-manage-view.png)

---

## ✨ Features

- **📱 Customer Digital Ordering**: QR code table session scanning, category filter tabs, item cart with special instructions, and real-time active order status tracking.
- **⚡ Real-Time SSE Event Stream**: Server-Sent Events (SSE) broadcasting instant status updates across Kitchen, Waiter, and Cashier terminals without page reloads.
- **👨‍🍳 Kitchen Queue & Status Selector**: Item-level status workflow (`Pending` ➔ `Preparing 🔥` ➔ `Finished ✅` ➔ `Served 🛎️` ➔ `Cancelled 🚫` for out-of-stock items) with filter tabs and misclick status revert capabilities.
- **🛎️ Waiter Order Tracking**: Filterable dish status dashboard allowing staff to track order progress and update or revert item statuses.
- **💳 Cashier Table & POS Checkout**: Active table management, order receipt summary, manual payment recording, and Stripe checkout integration.
- **🛠️ Admin Management**:
  - **User Management**: Create, update, deactivate, and reactivate staff user accounts.
  - **Table Setup & Management**: Configure table numbers, names, capacity, and active/inactive status toggle.
  - **Food Menu & Category Management**: Create and update menu items, category ordering, available/unavailable toggles, modal image uploads, and soft-delete (`deleted_at`).
- **💰 Robust Monetary Math**: All monetary values are handled in integer cents (e.g. `$12.50` = `1250`) to eliminate floating-point rounding errors across database, backend API, and Stripe integration.

---

## 🛠️ Tech Stack

### Backend
- **Framework**: Rust, Axum, Tokio async runtime
- **Database**: PostgreSQL with SQLx compile-time query verification & migrations
- **Authentication**: JWT authentication with Argon2 password hashing
- **Real-Time Streaming**: Server-Sent Events (SSE) via Tokio broadcast channels
- **Payments**: Stripe API integration

### Frontend
- **Framework**: Vue 3 (Composition API), TypeScript, Vite
- **State & Routing**: Pinia, Vue Router
- **Styling**: Vanilla CSS with dark mode

---

## 🚀 Development Setup

### Prerequisites
- **Rust**: `rustc` and `cargo` (edition 2024)
- **Node.js**: v25.7.0+ with `pnpm`
- **PostgreSQL**: PostgreSQL server running locally or via Docker

### Quick Start

1. **Clone & Install Dependencies**:
   ```bash
   pnpm --prefix web install
   ```

2. **Environment Variables**:
   Create `.env` in the repository root:
   ```env
   DATABASE_URL=postgres://postgres:example@localhost:5432/restaurant
   JWT_SECRET=super-secret-jwt-key
   STRIPE_SECRET_KEY=sk_test_...
   PORT=3000
   ```

3. **Database Initialization & Seeding**:
   Run database reset and seed script (seeds users, tables, categories, menu items, and copies demo dish images):
   ```bash
   ./db-reset.sh
   ```

4. **Run Development Server**:
   Start both backend (`cargo watch`) and frontend (`vite dev`) with watch mode:
   ```bash
   ./dev.sh
   ```

   - **Frontend App**: [http://localhost:5173](http://localhost:5173)
   - **Backend API**: [http://localhost:3000](http://localhost:3000)
   - **Adminer**: [http://localhost:9001](http://localhost:9001)

---

## 🔑 Demo Accounts

Default credentials for testing different role permissions (Password: `admin` or `password`):

| Role | Username | Password | Access Level |
| :--- | :--- | :--- | :--- |
| **Admin** | `admin` | `admin` | Full system configuration & management |
| **Cashier** | `cashier1` | `password` | POS checkout, receipts & table management |
| **Kitchen** | `kitchen1` | `password` | Kitchen queue & dish status updates |
| **Waiter** | `waiter1` | `password` | Waiter dashboard & order status tracking |
