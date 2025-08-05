-- migrations/YYYYMMDDHHMMSS_create_erikas_and_roles.sql

-- Stworzenie typu ENUM dla ról w systemie
CREATE TYPE user_role AS ENUM ('Admin', 'Erika', 'User');

-- Stworzenie tabeli dla profili modelek ("Erik")
CREATE TABLE erikas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role user_role NOT NULL DEFAULT 'Erika',
    is_approved BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
