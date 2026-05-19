-- Add user_id column to transactions table as nullable first
ALTER TABLE transactions ADD COLUMN user_id UUID;

-- Populate existing transactions with the user_id of their associated pocket
UPDATE transactions t
SET user_id = p.user_id
FROM pockets p
WHERE t.pocket_id = p.id;

-- Make user_id NOT NULL now that existing rows are populated
ALTER TABLE transactions ALTER COLUMN user_id SET NOT NULL;

-- Add index on user_id for faster lookups
CREATE INDEX IF NOT EXISTS idx_transactions_user_id ON transactions(user_id);
