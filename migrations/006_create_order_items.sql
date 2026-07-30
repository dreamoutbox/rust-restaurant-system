CREATE TABLE IF NOT EXISTS order_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    menu_item_id UUID NOT NULL REFERENCES menu_items(id) ON DELETE RESTRICT,
    quantity INT NOT NULL DEFAULT 1,
    unit_price BIGINT NOT NULL,
    note TEXT,
    status VARCHAR(20) NOT NULL CHECK (status IN ('pending', 'preparing', 'finished', 'served', 'cancelled')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
