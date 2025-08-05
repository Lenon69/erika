-- migrations/YYYY..._create_galleries_table.sql
CREATE TABLE galleries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    erika_id UUID NOT NULL REFERENCES erikas(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    price_pln DECIMAL(10, 2), -- Cena w PLN, np. 19.99
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
