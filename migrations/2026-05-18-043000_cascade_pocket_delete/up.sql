-- Drop existing foreign key constraints
ALTER TABLE transactions DROP CONSTRAINT IF EXISTS transactions_pocket_id_fkey;
ALTER TABLE transactions DROP CONSTRAINT IF EXISTS transactions_destination_pocket_id_fkey;

-- Recreate transactions foreign keys with ON DELETE CASCADE
ALTER TABLE transactions
    ADD CONSTRAINT transactions_pocket_id_fkey
    FOREIGN KEY (pocket_id) REFERENCES pockets(id) ON DELETE CASCADE;

ALTER TABLE transactions
    ADD CONSTRAINT transactions_destination_pocket_id_fkey
    FOREIGN KEY (destination_pocket_id) REFERENCES pockets(id) ON DELETE CASCADE;

-- Drop existing notification_inbox foreign key constraints
ALTER TABLE notification_inbox DROP CONSTRAINT IF EXISTS notification_inbox_pocket_id_fkey;
ALTER TABLE notification_inbox DROP CONSTRAINT IF EXISTS notification_inbox_destination_pocket_id_fkey;
ALTER TABLE notification_inbox DROP CONSTRAINT IF EXISTS notification_inbox_transaction_id_fkey;

-- Recreate notification_inbox foreign keys with ON DELETE SET NULL
ALTER TABLE notification_inbox
    ADD CONSTRAINT notification_inbox_pocket_id_fkey
    FOREIGN KEY (pocket_id) REFERENCES pockets(id) ON DELETE SET NULL;

ALTER TABLE notification_inbox
    ADD CONSTRAINT notification_inbox_destination_pocket_id_fkey
    FOREIGN KEY (destination_pocket_id) REFERENCES pockets(id) ON DELETE SET NULL;

ALTER TABLE notification_inbox
    ADD CONSTRAINT notification_inbox_transaction_id_fkey
    FOREIGN KEY (transaction_id) REFERENCES transactions(id) ON DELETE SET NULL;
