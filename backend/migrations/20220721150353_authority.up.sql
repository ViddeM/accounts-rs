CREATE TYPE AUTHORITY_LEVEL AS ENUM ('user', 'admin');
ALTER TABLE account
ADD COLUMN authority AUTHORITY_LEVEL NOT NULL DEFAULT 'user';