ALTER TABLE transactions ADD COLUMN status VARCHAR(20) DEFAULT 'resolved' NOT NULL CHECK (status IN ('pending', 'resolved', 'rejected'));
