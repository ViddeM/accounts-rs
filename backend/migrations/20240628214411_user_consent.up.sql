-- Add up migration script here
CREATE TABLE user_client_consent(
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES oauth_client(id),
    account_id UUID NOT NULL REFERENCES account(id),
    consented_on TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TYPE OAUTH_SCOPE AS ENUM ('email', 'openid');

CREATE TABLE client_scope(
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES oauth_client(id),
    scope OAUTH_SCOPE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE(client_id, scope)
);

CREATE TABLE user_client_consented_scope(
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_client_consent_id UUID NOT NULL REFERENCES user_client_consent(id),
    client_scope_id UUID NOT NULL REFERENCES client_scope(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE(user_client_consent_id, client_scope_id)
);
