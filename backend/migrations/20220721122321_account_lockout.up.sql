ALTER TABLE login_details
ADD COLUMN incorrect_password_count INT NOT NULL default 0;
ALTER TABLE login_details
ADD COLUMN account_locked_until TIMESTAMPTZ;