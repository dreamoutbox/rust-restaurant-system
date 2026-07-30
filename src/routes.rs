use axum::{
    Router,
    routing::{get, patch, post, put},
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};

use crate::handlers::{
    auth::{AppState, get_me, login, logout},
    category::{create_category, delete_category, list_categories, update_category},
    menu_item::{
        create_menu_item, delete_menu_item, list_all_menu_items, list_menu, update_menu_item,
        upload_menu_item_image,
    },
    order::{
        get_customer_order_status, get_customer_session_menu, get_order_detail, list_orders,
        submit_customer_order_items,
    },
    order_item::update_order_item_status,
    payment::{
        checkout_order, create_stripe_checkout_session, handle_stripe_webhook,
        record_manual_payment,
    },
    sse::sse_handler,
    table::{
        close_table, create_table, delete_table, get_table_qr, list_tables, open_table,
        update_table,
    },
    user::{create_user, delete_user, list_users, update_user},
};

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api_routes = Router::new()
        // Auth
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/auth/me", get(get_me))
        // Users (Admin)
        .route("/users", get(list_users).post(create_user))
        .route("/users/{id}", put(update_user).delete(delete_user))
        // Tables
        .route("/tables", get(list_tables).post(create_table))
        .route("/tables/{id}", put(update_table).delete(delete_table))
        .route("/tables/{id}/open", post(open_table))
        .route("/tables/{id}/close", post(close_table))
        .route("/tables/{id}/qr", get(get_table_qr))
        // Categories
        .route("/categories", get(list_categories).post(create_category))
        .route(
            "/categories/{id}",
            put(update_category).delete(delete_category),
        )
        // Menu Items
        .route("/menu", get(list_menu).post(create_menu_item))
        .route("/menu/all", get(list_all_menu_items))
        .route("/menu/{id}", put(update_menu_item).delete(delete_menu_item))
        .route("/menu/{id}/image", post(upload_menu_item_image))
        // Customer Public Routes
        .route("/order/{token}/menu", get(get_customer_session_menu))
        .route("/order/{token}/items", post(submit_customer_order_items))
        .route("/order/{token}/status", get(get_customer_order_status))
        // Staff Orders
        .route("/orders", get(list_orders))
        .route("/orders/{id}", get(get_order_detail))
        .route("/order-items/{id}/status", patch(update_order_item_status))
        // Payments
        .route("/orders/{id}/checkout", post(checkout_order))
        .route("/orders/{id}/pay/manual", post(record_manual_payment))
        .route(
            "/orders/{id}/pay/stripe",
            post(create_stripe_checkout_session),
        )
        .route("/webhooks/stripe", post(handle_stripe_webhook))
        // SSE Real-time Events
        .route("/events", get(sse_handler))
        .with_state(state.clone());

    let mut router = Router::new()
        .nest("/api", api_routes)
        .nest_service("/uploads", ServeDir::new(&state.config.upload_dir));

    let static_dir = std::path::Path::new("web/dist");
    if static_dir.exists() {
        let serve_spa =
            ServeDir::new(static_dir).fallback(ServeFile::new(static_dir.join("index.html")));
        router = router.fallback_service(serve_spa);
    }

    router.layer(cors)
}
