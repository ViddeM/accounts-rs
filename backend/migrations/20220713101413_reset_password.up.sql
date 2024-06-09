CREATE TABLE password_reset (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    login_details UUID NOT NULL REFERENCES login_details UNIQUE,
    code UUID NOT NULL DEFAULT gen_random_uuid() UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    modified_at TIMESTAMPTZ NOT NULL DEFAULT now()
);