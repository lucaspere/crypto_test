-- migrate:up
CREATE SCHEMA IF NOT EXISTS social;

-- Table: social.groups
CREATE TABLE IF NOT EXISTS social.groups (
    id BIGSERIAL PRIMARY KEY,
    name character varying(255) NOT NULL,
    logo_uri character varying(255),
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS social.group_users (
    group_id bigint,
    user_id uuid,
    joined_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT group_users_pkey PRIMARY KEY (group_id, user_id)
);


-- Table: social.tokens
CREATE TABLE IF NOT EXISTS social.tokens (
    address character varying(255) NOT NULL,
    name character varying(255) NOT NULL,
    symbol character varying(50) NOT NULL,
    chain character varying(50) NOT NULL,
    CONSTRAINT tokens_pkey PRIMARY KEY (address, chain)
);

-- Table: social.token_picks
CREATE TABLE IF NOT EXISTS social.token_picks (
    id BIGSERIAL PRIMARY KEY,
    token_address character varying(255),
    user_id uuid,
    group_id bigint,
    telegram_message_id bigint,
    price_at_call numeric(18,8) NOT NULL,
    market_cap_at_call numeric(18,8),
    supply_at_call numeric(18,8),
    highest_market_cap numeric(18,8),
    hit_date timestamp with time zone,
    call_date timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT token_picks_group_id_fkey FOREIGN KEY (group_id)
        REFERENCES social.groups (id) ON DELETE CASCADE
);

-- Table: social.comments
CREATE TABLE IF NOT EXISTS social.comments (
    id BIGSERIAL PRIMARY KEY,
    token_pick_id bigint,
    user_id uuid,
    content character varying(500) NOT NULL,
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT comments_token_pick_id_fkey FOREIGN KEY (token_pick_id)
        REFERENCES social.token_picks (id) ON DELETE CASCADE
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

-- Enum: social.tier_type
CREATE TYPE social.tier_type AS ENUM ('iron', 'bronze', 'silver', 'gold', 'platinum', 'emerald', 'diamond');

-- Table: social.user_tiers
CREATE TABLE IF NOT EXISTS social.user_tiers (
    user_id uuid NOT NULL,
    tier social.tier_type NOT NULL,
    earned_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT user_tiers_pkey PRIMARY KEY (user_id, tier)
);

-- Table: social.point_transactions
CREATE TABLE IF NOT EXISTS social.point_transactions (
    id BIGSERIAL PRIMARY KEY,
    user_id uuid,
    points_earned bigint NOT NULL,
    action_type character varying(50) NOT NULL,
    context character varying(50) NOT NULL,
    details jsonb NOT NULL,
    created_at timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
);



-- Create useful indexes
CREATE INDEX idx_token_picks_user_id ON social.token_picks(user_id);
CREATE INDEX idx_token_picks_token_address ON social.token_picks(token_address);
CREATE INDEX idx_token_picks_call_date ON social.token_picks(call_date);

CREATE INDEX idx_user_follows_follower_id ON social.user_follows(follower_id);
CREATE INDEX idx_user_follows_followed_id ON social.user_follows(followed_id);

CREATE INDEX idx_user_tiers_user_id ON social.user_tiers(user_id);
CREATE INDEX idx_user_tiers_tier ON social.user_tiers(tier);

CREATE INDEX idx_point_transactions_user_id ON social.point_transactions(user_id);
CREATE INDEX idx_point_transactions_action_type ON social.point_transactions(action_type);
CREATE INDEX idx_point_transactions_created_at ON social.point_transactions(created_at);

CREATE INDEX idx_comments_token_pick_id ON social.comments(token_pick_id);
CREATE INDEX idx_comments_user_id ON social.comments(user_id);
CREATE INDEX idx_comments_created_at ON social.comments(created_at);

CREATE INDEX idx_group_users_group_id ON social.group_users(group_id);
CREATE INDEX idx_group_users_user_id ON social.group_users(user_id);

-- migrate:down

DROP INDEX IF EXISTS idx_token_picks_user_id;
DROP INDEX IF EXISTS idx_token_picks_token_address;
DROP INDEX IF EXISTS idx_token_picks_call_date;
DROP INDEX IF EXISTS idx_user_follows_follower_id;
DROP INDEX IF EXISTS idx_user_follows_followed_id;
DROP INDEX IF EXISTS idx_user_tiers_user_id;
DROP INDEX IF EXISTS idx_user_tiers_tier;
DROP INDEX IF EXISTS idx_point_transactions_user_id;
DROP INDEX IF EXISTS idx_point_transactions_action_type;
DROP INDEX IF EXISTS idx_point_transactions_created_at;
DROP INDEX IF EXISTS idx_comments_token_pick_id;
DROP INDEX IF EXISTS idx_comments_user_id;
DROP INDEX IF EXISTS idx_comments_created_at;

DROP TABLE IF EXISTS social.comments;
DROP TABLE IF EXISTS social.tokens;
DROP TABLE IF EXISTS social.token_picks;
DROP TABLE IF EXISTS social.user_points;
DROP TABLE IF EXISTS social.user_follows;
DROP TABLE IF EXISTS social.user_tiers;
DROP TABLE IF EXISTS social.point_transactions;
DROP TABLE IF EXISTS social.groups;
DROP TABLE IF EXISTS social.group_users;

DROP TYPE IF EXISTS social.tier_type;
