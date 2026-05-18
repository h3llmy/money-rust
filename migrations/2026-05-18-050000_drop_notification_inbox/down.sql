CREATE TABLE notification_inbox (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    app_package VARCHAR(150) NOT NULL,
    raw_title TEXT,
    raw_body TEXT NOT NULL,
    received_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    status VARCHAR(20) DEFAULT 'pending' NOT NULL CHECK (status IN ('pending', 'processed', 'failed', 'ignored')),
    transaction_id UUID CONSTRAINT notification_inbox_transaction_id_fkey REFERENCES transactions(id) ON DELETE SET NULL,
    amount NUMERIC(15, 2),
    type VARCHAR(20) CHECK (type IN ('income', 'expense', 'transfer')),
    pocket_id UUID CONSTRAINT notification_inbox_pocket_id_fkey REFERENCES pockets(id) ON DELETE SET NULL,
    category_id UUID REFERENCES categories(id),
    destination_pocket_id UUID CONSTRAINT notification_inbox_destination_pocket_id_fkey REFERENCES pockets(id) ON DELETE SET NULL,
    title VARCHAR(255)
);
