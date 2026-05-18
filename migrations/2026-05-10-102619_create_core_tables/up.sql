-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- 1. categories
CREATE TABLE categories (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID,
    name VARCHAR(100) NOT NULL,
    type VARCHAR(20) NOT NULL CHECK (type IN ('income', 'expense'))
);

-- 2. pockets
CREATE TABLE pockets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL,
    name VARCHAR(100) NOT NULL,
    pocket_type VARCHAR(50) NOT NULL,
    currency VARCHAR(3) DEFAULT 'IDR' NOT NULL,
    balance NUMERIC(15, 2) NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- 3. transactions
CREATE TABLE transactions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    pocket_id UUID NOT NULL REFERENCES pockets(id),
    category_id UUID REFERENCES categories(id),
    amount NUMERIC(15, 2) NOT NULL,
    type VARCHAR(20) NOT NULL CHECK (type IN ('income', 'expense', 'transfer')),
    title VARCHAR(255) NOT NULL,
    transaction_time TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    destination_pocket_id UUID REFERENCES pockets(id)
);

-- 4. notification_inbox
CREATE TABLE notification_inbox (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    app_package VARCHAR(150) NOT NULL,
    raw_title TEXT,
    raw_body TEXT NOT NULL,
    received_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    status VARCHAR(20) DEFAULT 'pending' NOT NULL CHECK (status IN ('pending', 'processed', 'failed', 'ignored')),
    transaction_id UUID REFERENCES transactions(id),
    amount NUMERIC(15, 2),
    type VARCHAR(20) CHECK (type IN ('income', 'expense', 'transfer')),
    pocket_id UUID REFERENCES pockets(id),
    category_id UUID REFERENCES categories(id),
    destination_pocket_id UUID REFERENCES pockets(id),
    title VARCHAR(255)
);