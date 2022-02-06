CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- An account, every user has one of these.
CREATE TABLE account (
     id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
     first_name TEXT NOT NULL,
     last_name TEXT NOT NULL,
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     modified_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Login details for an account, required to login directly on the site (without any third party).
CREATE TABLE login_details (
    account_id UUID PRIMARY KEY REFERENCES account(id),
    email TEXT UNIQUE NOT NULL,
    password TEXT NOT NULL,
    password_nonces TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    modified_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- A third-paty login provider such as Google, Facebook, Github etc.
CREATE TABLE login_provider (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    modified_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Login via a third party login provider
CREATE TABLE third_party_login (
    account_id UUID PRIMARY KEY REFERENCES account(id),
    login_provider_id UUID NOT NULL REFERENCES login_provider(id),
    email TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    modified_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- The whitelist table
CREATE TABLE whitelist (
    email TEXT NOT NULL,
    -- If login_provider is null it refers to login_details
    login_provider_id UUID REFERENCES login_provider(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    modified_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (email, login_provider_id)
)