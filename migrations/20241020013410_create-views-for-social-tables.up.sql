-- Add up migration script here


-- View for tokens
CREATE OR REPLACE VIEW social.v_tokens AS
SELECT
    address,
    name,
    symbol,
    created_at,
    updated_at
FROM
    social.tokens;

-- View for token_calls
CREATE OR REPLACE VIEW v_token_calls AS
SELECT
    tc.id,
    tc.token_address,
    t.name AS token_name,
    t.symbol AS token_symbol,
    tc.user_id,
    u.username AS user_username,
    tc.call_type,
    tc.price_at_call,
    tc.target_price,
    tc.call_date
FROM
    social.token_calls tc
JOIN
    social.tokens t ON tc.token_address = t.address
JOIN
    users u ON tc.user_id = u.id;

-- View for user_follows
CREATE OR REPLACE VIEW social.v_user_follows AS
SELECT
    uf.follower_id,
    f.username AS follower_username,
    uf.followed_id,
    fd.username AS followed_username,
    uf.created_at
FROM
    social.user_follows uf
JOIN
    users f ON uf.follower_id = f.id
JOIN
    users fd ON uf.followed_id = fd.id;

-- View for user statistics
CREATE OR REPLACE VIEW social.v_user_stats AS
SELECT
    u.id AS user_id,
    u.username,
    COUNT(DISTINCT tc.id) AS total_token_calls,
    COUNT(DISTINCT CASE WHEN tc.call_type = 'buy' THEN tc.id END) AS buy_calls,
    COUNT(DISTINCT CASE WHEN tc.call_type = 'sell' THEN tc.id END) AS sell_calls,
    COUNT(DISTINCT f.followed_id) AS following_count,
    COUNT(DISTINCT f2.follower_id) AS followers_count
FROM
    users u
LEFT JOIN
    social.token_calls tc ON u.id = tc.user_id
LEFT JOIN
    social.user_follows f ON u.id = f.follower_id
LEFT JOIN
    social.user_follows f2 ON u.id = f2.followed_id
GROUP BY
    u.id, u.username;
