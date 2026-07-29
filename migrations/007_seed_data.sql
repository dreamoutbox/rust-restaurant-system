-- Seed Users
-- Passwords: admin -> 'admin', cashier1/kitchen1/waiter1 -> 'password'
-- Argon2id hashes:
-- 'admin' -> $argon2id$v=19$m=19456,t=2,p=1$c2FsdHNhbHRzYWx0c2FsdA$l37YVbvh+o0S9dE4X2a/j8v1w9k2x1L5m
-- 'password' -> $argon2id$v=19$m=19456,t=2,p=1$c2FsdHNhbHRzYWx0c2FsdA$9eK1Z/p010x156s+W5q32n7k/X19s+0
-- We will also handle auto-seeding/upgrading users in Rust code on startup to ensure password hash validity.

INSERT INTO users (id, username, password_hash, display_name, role) VALUES
    ('00000000-0000-0000-0000-000000000001', 'admin', '$argon2id$v=19$m=19456,t=2,p=1$Z9K6WbZwgGhC5dgtkNBoLA$hzyyO2HS/vBHpJdTzrGZOGKH7B6flwzAYxgzBsF4p10', 'Admin User', 'admin'),
    ('00000000-0000-0000-0000-000000000002', 'cashier1', '$argon2id$v=19$m=19456,t=2,p=1$p9LTw1QwQcz3lkrPxLRDyQ$0E6VAreKUp++6aIxzYBKojUQ3a6D/c/XUksWYW+9awI', 'Cashier 1', 'cashier'),
    ('00000000-0000-0000-0000-000000000003', 'kitchen1', '$argon2id$v=19$m=19456,t=2,p=1$p9LTw1QwQcz3lkrPxLRDyQ$0E6VAreKUp++6aIxzYBKojUQ3a6D/c/XUksWYW+9awI', 'Chef Alex', 'kitchen'),
    ('00000000-0000-0000-0000-000000000004', 'waiter1', '$argon2id$v=19$m=19456,t=2,p=1$p9LTw1QwQcz3lkrPxLRDyQ$0E6VAreKUp++6aIxzYBKojUQ3a6D/c/XUksWYW+9awI', 'Waiter John', 'waiter')
ON CONFLICT (username) DO NOTHING;

-- Seed Tables (Tables 1 - 8)
INSERT INTO tables (table_number, name, capacity) VALUES
    (1, 'Table 1 (Indoor)', 4),
    (2, 'Table 2 (Indoor)', 4),
    (3, 'Table 3 (Window Seat)', 2),
    (4, 'Table 4 (Window Seat)', 2),
    (5, 'Table 5 (Patio)', 6),
    (6, 'Table 6 (Patio)', 6),
    (7, 'VIP Booth A', 8),
    (8, 'VIP Booth B', 8)
ON CONFLICT (table_number) DO NOTHING;

-- Seed Categories
INSERT INTO categories (id, name, sort_order) VALUES
    ('10000000-0000-0000-0000-000000000001', 'Appetizers', 1),
    ('10000000-0000-0000-0000-000000000002', 'Main Course', 2),
    ('10000000-0000-0000-0000-000000000003', 'Desserts', 3),
    ('10000000-0000-0000-0000-000000000004', 'Beverages', 4)
ON CONFLICT (name) DO NOTHING;

-- Seed Menu Items
INSERT INTO menu_items (category_id, name, description, price, sort_order) VALUES
    ('10000000-0000-0000-0000-000000000001', 'Crispy Spring Rolls', 'Vegetable spring rolls served with sweet chili sauce', 6.50, 1),
    ('10000000-0000-0000-0000-000000000001', 'Garlic Butter Shrimp', 'Sautéed shrimp with garlic, white wine, and fresh parsley', 9.90, 2),
    ('10000000-0000-0000-0000-000000000001', 'Caesar Salad', 'Romaine lettuce, parmesan, croutons, and Caesar dressing', 7.50, 3),
    ('10000000-0000-0000-0000-000000000002', 'Grilled Ribeye Steak', '300g ribeye steak with truffle butter and mashed potatoes', 24.90, 1),
    ('10000000-0000-0000-0000-000000000002', 'Classic Cheese Burger', 'Angus beef patty, cheddar, lettuce, tomato, pickles, and fries', 14.50, 2),
    ('10000000-0000-0000-0000-000000000002', 'Creamy Carbonara Pasta', 'Spaghetti with guanciale, egg yolk, pecorino cheese, and black pepper', 13.00, 3),
    ('10000000-0000-0000-0000-000000000002', 'Grilled Salmon Filet', 'Atlantic salmon with lemon herb sauce and roasted asparagus', 19.50, 4),
    ('10000000-0000-0000-0000-000000000003', 'Tiramisu', 'Classic Italian coffee-flavored dessert', 6.50, 1),
    ('10000000-0000-0000-0000-000000000003', 'Chocolate Lava Cake', 'Warm chocolate cake with a molten center and vanilla ice cream', 7.50, 2),
    ('10000000-0000-0000-0000-000000000004', 'Iced Matcha Latte', 'Japanese green tea with fresh milk', 4.50, 1),
    ('10000000-0000-0000-0000-000000000004', 'Fresh Lemonade', 'House-made sparkling lemonade', 3.80, 2),
    ('10000000-0000-0000-0000-000000000004', 'Craft Pale Ale', 'Local craft beer 500ml', 6.00, 3)
ON CONFLICT DO NOTHING;
