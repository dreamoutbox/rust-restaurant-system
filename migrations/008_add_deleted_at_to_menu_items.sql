-- Add soft delete timestamp to menu_items
ALTER TABLE menu_items ADD COLUMN deleted_at TIMESTAMPTZ;
