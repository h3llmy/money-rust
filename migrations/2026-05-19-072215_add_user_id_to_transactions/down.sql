DROP INDEX IF EXISTS idx_transactions_user_id;
ALTER TABLE transactions DROP COLUMN user_id;
