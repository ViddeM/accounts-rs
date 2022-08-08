-- POSTGRES MOCK DATA
INSERT INTO login_provider(name)
VALUES ('local');
-- Insert admin account with email: 'pelle.karlsson@test.com' and password: 'asdasd123123'
INSERT INTO whitelist(email, login_provider)
VALUES ('pelle.karlsson@test.com', 'local');
INSERT INTO account(id, first_name, last_name, authority)
VALUES (
        'a87a5aa2-b4dd-484e-854f-9db54b119d87',
        'Pelle',
        'Karlsson',
        'admin'
    );
INSERT INTO login_details(
        account_id,
        email,
        activated_at,
        password,
        password_nonces
    )
VALUES (
        'a87a5aa2-b4dd-484e-854f-9db54b119d87',
        'pelle.karlsson@test.com',
        '1990-01-01',
        '354AF1C88A9BBBC6B6E1E2A9E67FB6F7B25739A9284A5C1861619151C62FFDA0B571DF34939C00D7C32E2D6FD5F753919A241914F08F23923F628FA71900BE1536DEBC4AB1E69D9F2EAF5A58983CC9087126A5DB7350DAF4BFA97A527716A1F658CC0655A8340954D2E74DBF210B62BFD4',
        'DF69D5A22F0B0178D3561346'
    );
-- Insert standard account with email: 'karl.larsson@pleasedontbeavaliddomain.com' and password 'password123'
INSERT INTO whitelist(email, login_provider)
VALUES (
        'karl.larsson@pleasedontbeavaliddomain.com',
        'local'
    );
INSERT INTO account(id, first_name, last_name)
VALUES (
        '4cbeabf3-725b-40af-8d04-2b61ca9534b4',
        'Karl',
        'Larsson'
    );
INSERT INTO login_details(
        account_id,
        email,
        activated_at,
        password,
        password_nonces
    )
VALUES(
        '4cbeabf3-725b-40af-8d04-2b61ca9534b4',
        'karl.larsson@pleasedontbeavaliddomain.com',
        '2022-08-08 10:49:10.48415+00',
        '6B2BB4DC9D60E687E63787024A7DB306C3B0FFF7B0A938C4E19C4E79E4AE48A3A59462A12DEA5EC874F0205C71440EF229CFBEDA0C1480C13781FC7CD2F1C91014E888813B5C04006E65808481221CE70D08D89921FD99EC3E13D813AA1BC70920BD907576C7A7E6C2065313B313B9D838',
        '8316690366BFA8B8CE670987'
    );