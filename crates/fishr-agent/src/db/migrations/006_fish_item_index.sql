-- 006_fish_item_index.sql
-- Add index on sale_item.fish_item_id for query performance
-- Note: FK constraint (REFERENCES fish_item(id)) cannot be added via ALTER TABLE in SQLite.
-- A table rebuild would be required. The index mitigates the most common performance concern.

CREATE INDEX IF NOT EXISTS idx_sale_item_fish ON sale_item(fish_item_id);
