ALTER TABLE login_details
ADD COLUMN activated BOOL NOT NULL
DEFAULT false;

CREATE TABLE activation_code (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    login_details UUID NOT NULL REFERENCES login_details UNIQUE,
    code UUID NOT NULL DEFAULT uuid_generate_v4() UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    modified_at TIMESTAMPTZ NOT NULL DEFAULT now()
)