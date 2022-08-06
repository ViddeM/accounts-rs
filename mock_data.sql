-- POSTGRES MOCK DATA

-- Insert account with email: 'pelle.karlsson@test.com' and password: 'asdasd123123'
INSERT INTO login_provider(name) VALUES ('local');
INSERT INTO whitelist(email, login_provider) VALUES ('pelle.karlsson@test.com', 'local');
INSERT INTO account(id, first_name, last_name) VALUES ('a87a5aa2-b4dd-484e-854f-9db54b119d87', 'Pelle', 'Karlsson');
INSERT INTO login_details(account_id, email, activated_at, password, password_nonces) VALUES ('a87a5aa2-b4dd-484e-854f-9db54b119d87', 'pelle.karlsson@test.com', '1990-01-01', '354AF1C88A9BBBC6B6E1E2A9E67FB6F7B25739A9284A5C1861619151C62FFDA0B571DF34939C00D7C32E2D6FD5F753919A241914F08F23923F628FA71900BE1536DEBC4AB1E69D9F2EAF5A58983CC9087126A5DB7350DAF4BFA97A527716A1F658CC0655A8340954D2E74DBF210B62BFD4', 'DF69D5A22F0B0178D3561346'); 

