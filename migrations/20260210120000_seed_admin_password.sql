-- Update or insert admin user with secure default password
-- Password: ZO6gOCn0icxcvrke62F96A==
INSERT INTO users (id, username, password_hash)
VALUES ('00000000-0000-0000-0000-000000000000', 'admin', '$argon2id$v=19$m=19456,t=2,p=1$Ewiz6jCZu9NGQaAJtWRLqg$Fn5yB19PZG+eTq/f1oKbw+tsqvhwuAnMI3TpQCIg9vI')
ON CONFLICT (username)
DO UPDATE SET password_hash = EXCLUDED.password_hash;
