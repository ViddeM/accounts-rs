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

CREATE TABLE login_provider (
    name TEXT PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    modified_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Login via a third party login provider
CREATE TABLE third_party_login (
    account_id UUID PRIMARY KEY REFERENCES account(id),
    login_provider TEXT NOT NULL REFERENCES login_provider(name),
    email TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    modified_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- The whitelist table
CREATE TABLE whitelist (
    email TEXT NOT NULL,
    login_provider TEXT REFERENCES login_provider(name),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    modified_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (email, login_provider)
);