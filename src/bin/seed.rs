use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::fs;
use std::path::Path;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:example@localhost:5432/restaurant".to_string());

    println!("Connecting to database for seeding: {}", database_url);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Ensure ./uploads directory exists
    let uploads_dir = Path::new("./uploads");
    if !uploads_dir.exists() {
        fs::create_dir_all(uploads_dir)?;
    }

    // 1. Seed Users
    println!("Seeding Users...");
    let argon2 = Argon2::default();

    let users_data = [
        (
            "00000000-0000-0000-0000-000000000001",
            "admin",
            "admin",
            "Admin User",
            "admin",
        ),
        (
            "00000000-0000-0000-0000-000000000002",
            "cashier1",
            "password",
            "Cashier 1",
            "cashier",
        ),
        (
            "00000000-0000-0000-0000-000000000003",
            "kitchen1",
            "password",
            "Chef Alex",
            "kitchen",
        ),
        (
            "00000000-0000-0000-0000-000000000004",
            "waiter1",
            "password",
            "Waiter John",
            "waiter",
        ),
    ];

    for (id_str, username, plain_pw, display_name, role) in users_data {
        let id = Uuid::parse_str(id_str)?;
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = argon2
            .hash_password(plain_pw.as_bytes(), &salt)
            .unwrap()
            .to_string();

        sqlx::query!(
            r#"
            INSERT INTO users (id, username, password_hash, display_name, role)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (username) DO NOTHING
            "#,
            id,
            username,
            password_hash,
            display_name,
            role
        )
        .execute(&pool)
        .await?;
    }

    // 2. Seed Tables
    println!("Seeding Tables...");
    let tables_data = [
        (1, "Table 1 (Indoor)", 4),
        (2, "Table 2 (Indoor)", 4),
        (3, "Table 3 (Window Seat)", 2),
        (4, "Table 4 (Window Seat)", 2),
        (5, "Table 5 (Patio)", 6),
        (6, "Table 6 (Patio)", 6),
        (7, "VIP Booth A", 8),
        (8, "VIP Booth B", 8),
    ];

    for (table_number, name, capacity) in tables_data {
        sqlx::query!(
            r#"
            INSERT INTO tables (table_number, name, capacity)
            VALUES ($1, $2, $3)
            ON CONFLICT (table_number) DO NOTHING
            "#,
            table_number,
            name,
            capacity
        )
        .execute(&pool)
        .await?;
    }

    // 3. Seed Categories
    println!("Seeding Categories...");
    let categories_data = [
        ("10000000-0000-0000-0000-000000000001", "Appetizers", 1),
        ("10000000-0000-0000-0000-000000000002", "Main Course", 2),
        ("10000000-0000-0000-0000-000000000003", "Desserts", 3),
        ("10000000-0000-0000-0000-000000000004", "Beverages", 4),
    ];

    for (id_str, name, sort_order) in categories_data {
        let id = Uuid::parse_str(id_str)?;
        sqlx::query!(
            r#"
            INSERT INTO categories (id, name, sort_order)
            VALUES ($1, $2, $3)
            ON CONFLICT (name) DO NOTHING
            "#,
            id,
            name,
            sort_order
        )
        .execute(&pool)
        .await?;
    }

    // 4. Seed Menu Items & Copy Demo Images
    println!("Seeding Menu Items with Images...");
    let menu_items_data = [
        (
            "10000000-0000-0000-0000-000000000001",
            "Crispy Spring Rolls",
            "Vegetable spring rolls served with sweet chili sauce",
            650i64,
            1,
        ),
        (
            "10000000-0000-0000-0000-000000000001",
            "Garlic Butter Shrimp",
            "Sautéed shrimp with garlic, white wine, and fresh parsley",
            990i64,
            2,
        ),
        (
            "10000000-0000-0000-0000-000000000001",
            "Caesar Salad",
            "Romaine lettuce, parmesan, croutons, and Caesar dressing",
            750i64,
            3,
        ),
        (
            "10000000-0000-0000-0000-000000000002",
            "Grilled Ribeye Steak",
            "300g ribeye steak with truffle butter and mashed potatoes",
            2490i64,
            1,
        ),
        (
            "10000000-0000-0000-0000-000000000002",
            "Classic Cheese Burger",
            "Angus beef patty, cheddar, lettuce, tomato, pickles, and fries",
            1450i64,
            2,
        ),
        (
            "10000000-0000-0000-0000-000000000002",
            "Creamy Carbonara Pasta",
            "Spaghetti with guanciale, egg yolk, pecorino cheese, and black pepper",
            1300i64,
            3,
        ),
        (
            "10000000-0000-0000-0000-000000000002",
            "Grilled Salmon Filet",
            "Atlantic salmon with lemon herb sauce and roasted asparagus",
            1950i64,
            4,
        ),
        (
            "10000000-0000-0000-0000-000000000003",
            "Tiramisu",
            "Classic Italian coffee-flavored dessert",
            650i64,
            1,
        ),
        (
            "10000000-0000-0000-0000-000000000003",
            "Chocolate Lava Cake",
            "Warm chocolate cake with a molten center and vanilla ice cream",
            750i64,
            2,
        ),
        (
            "10000000-0000-0000-0000-000000000004",
            "Iced Matcha Latte",
            "Japanese green tea with fresh milk",
            450i64,
            1,
        ),
        (
            "10000000-0000-0000-0000-000000000004",
            "Fresh Lemonade",
            "House-made sparkling lemonade",
            380i64,
            2,
        ),
        (
            "10000000-0000-0000-0000-000000000004",
            "Craft Pale Ale",
            "Local craft beer 500ml",
            600i64,
            3,
        ),
    ];

    for (cat_id_str, name, description, price, sort_order) in menu_items_data {
        let category_id = Uuid::parse_str(cat_id_str)?;
        let src_img = format!("demo/images/{}.jpg", name);
        let mut image_path: Option<String> = None;

        if Path::new(&src_img).exists() {
            let dest_file_name = format!("{}.jpg", name.replace(' ', "_"));
            let dest_path = uploads_dir.join(&dest_file_name);
            if let Err(e) = fs::copy(&src_img, &dest_path) {
                println!("Warning: Failed to copy image for {}: {}", name, e);
            } else {
                image_path = Some(format!("/uploads/{}", dest_file_name));
            }
        }

        sqlx::query!(
            r#"
            INSERT INTO menu_items (category_id, name, description, price, image_path, sort_order)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT DO NOTHING
            "#,
            category_id,
            name,
            description,
            price,
            image_path,
            sort_order
        )
        .execute(&pool)
        .await?;

        // Update image_path if conflict hit or existing item
        if let Some(ref img_p) = image_path {
            sqlx::query!(
                r#"
                UPDATE menu_items SET image_path = $1 WHERE category_id = $2 AND name = $3
                "#,
                img_p,
                category_id,
                name
            )
            .execute(&pool)
            .await?;
        }
    }

    println!("✅ Database seeding with demo images completed successfully!");
    Ok(())
}
