-- migrate:up
CREATE SCHEMA IF NOT EXISTS social;

CREATE TABLE IF NOT EXISTS users (
    id uuid PRIMARY KEY,
    username character varying(255) NOT NULL,
    telegram_id character varying(255) NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);

-- Table: social.groups
CREATE TABLE IF NOT EXISTS social.groups (
    id SERIAL PRIMARY KEY,
    name character varying(255) NOT NULL,
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS social.group_users (
    group_id bigint,
    user_id uuid,
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT group_users_pkey PRIMARY KEY (group_id, user_id)
);


-- Table: social.tokens
CREATE TABLE IF NOT EXISTS social.tokens (
    address character varying(255) NOT NULL,
    name character varying(255) NOT NULL,
    symbol character varying(50) NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT tokens_pkey PRIMARY KEY (address)
);

-- Table: social.token_calls
CREATE TABLE IF NOT EXISTS social.token_calls (
    id SERIAL PRIMARY KEY,
    token_address character varying(255),
    user_id uuid,
    group_id bigint,
    call_type character varying(50) NOT NULL,
    price_at_call numeric(18,8) NOT NULL,
    target_price numeric(18,8),
    call_date timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT token_calls_token_address_fkey FOREIGN KEY (token_address)
        REFERENCES social.tokens (address) ON DELETE CASCADE,
    CONSTRAINT token_calls_group_id_fkey FOREIGN KEY (group_id)
        REFERENCES social.groups (id) ON DELETE CASCADE
);

-- Table: social.comments
CREATE TABLE IF NOT EXISTS social.comments (
    id SERIAL PRIMARY KEY,
    token_call_id bigint,
    user_id uuid,
    content character varying(500) NOT NULL,
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT comments_token_call_id_fkey FOREIGN KEY (token_call_id)
        REFERENCES social.token_calls (id) ON DELETE CASCADE
);


-- Table: social.user_points
CREATE TABLE IF NOT EXISTS social.user_points (
    user_id uuid PRIMARY KEY,
    total_points bigint NOT NULL DEFAULT 0,
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Table: social.user_follows
CREATE TABLE IF NOT EXISTS social.user_follows (
    follower_id uuid NOT NULL,
    followed_id uuid NOT NULL,
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT user_follows_pkey PRIMARY KEY (follower_id, followed_id),
    CONSTRAINT user_follows_check CHECK (follower_id <> followed_id)
);

-- Enum: social.medal_type
CREATE TYPE social.medal_type AS ENUM ('iron', 'bronze', 'silver', 'gold');

-- Table: social.user_medals
CREATE TABLE IF NOT EXISTS social.user_medals (
    user_id uuid NOT NULL,
    medal_type social.medal_type NOT NULL,
    earned_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT user_medals_pkey PRIMARY KEY (user_id, medal_type)
);

-- Table: social.point_transactions
CREATE TABLE IF NOT EXISTS social.point_transactions (
    id SERIAL PRIMARY KEY,
    user_id uuid,
    points_earned bigint NOT NULL,
    action_type character varying(50) NOT NULL,
    context character varying(50) NOT NULL,
    details jsonb NOT NULL,
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
);



-- Create useful indexes
CREATE INDEX idx_token_calls_user_id ON social.token_calls(user_id);
CREATE INDEX idx_token_calls_token_address ON social.token_calls(token_address);
CREATE INDEX idx_token_calls_call_date ON social.token_calls(call_date);

CREATE INDEX idx_user_follows_follower_id ON social.user_follows(follower_id);
CREATE INDEX idx_user_follows_followed_id ON social.user_follows(followed_id);

CREATE INDEX idx_user_medals_user_id ON social.user_medals(user_id);
CREATE INDEX idx_user_medals_medal_type ON social.user_medals(medal_type);

CREATE INDEX idx_point_transactions_user_id ON social.point_transactions(user_id);
CREATE INDEX idx_point_transactions_action_type ON social.point_transactions(action_type);
CREATE INDEX idx_point_transactions_created_at ON social.point_transactions(created_at);

CREATE INDEX idx_comments_token_call_id ON social.comments(token_call_id);
CREATE INDEX idx_comments_user_id ON social.comments(user_id);
CREATE INDEX idx_comments_created_at ON social.comments(created_at);


-- migrate:down
-- Add down migration script here
DROP SCHEMA IF EXISTS social CASCADE;

DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS social.tokens;
DROP TABLE IF EXISTS social.token_calls;
DROP TABLE IF EXISTS social.user_points;
DROP TABLE IF EXISTS social.user_follows;
DROP TABLE IF EXISTS social.user_medals;
DROP TABLE IF EXISTS social.point_transactions;
DROP TABLE IF EXISTS social.comments;
DROP TABLE IF EXISTS social.groups;
DROP TABLE IF EXISTS social.group_users;
