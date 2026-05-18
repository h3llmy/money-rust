-- Drop cascade foreign key constraints
ALTER TABLE transactions DROP CONSTRAINT IF EXISTS transactions_pocket_id_fkey;
ALTER TABLE transactions DROP CONSTRAINT IF EXISTS transactions_destination_pocket_id_fkey;

-- Recreate standard transactions foreign keys
ALTER TABLE transactions
    ADD CONSTRAINT transactions_pocket_id_fkey
    FOREIGN KEY (pocket_id) REFERENCES pockets(id);

ALTER TABLE transactions
    ADD CONSTRAINT transactions_destination_pocket_id_fkey
    FOREIGN KEY (destination_pocket_id) REFERENCES pockets(id);

-- Drop set-null notification_inbox foreign key constraints
ALTER TABLE notification_inbox DROP CONSTRAINT IF EXISTS notification_inbox_pocket_id_fkey;
ALTER TABLE notification_inbox DROP CONSTRAINT IF EXISTS notification_inbox_destination_pocket_id_fkey;
ALTER TABLE notification_inbox DROP CONSTRAINT IF EXISTS notification_inbox_transaction_id_fkey;

-- Recreate standard notification_inbox foreign keys
ALTER TABLE notification_inbox
    ADD CONSTRAINT notification_inbox_pocket_id_fkey
    FOREIGN KEY (pocket_id) REFERENCES pockets(id);

ALTER TABLE notification_inbox
    ADD CONSTRAINT notification_inbox_destination_pocket_id_fkey
    FOREIGN KEY (destination_pocket_id) REFERENCES pockets(id);

ALTER TABLE notification_inbox
    ADD CONSTRAINT notification_inbox_transaction_id_fkey
    FOREIGN KEY (transaction_id) REFERENCES transactions(id);
