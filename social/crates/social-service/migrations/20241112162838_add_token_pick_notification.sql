-- migrate:up
CREATE OR REPLACE FUNCTION social.notify_token_pick()
RETURNS trigger AS $$
DECLARE
    pick_data jsonb;
    token_data jsonb;
    user_data jsonb;
	group_data jsonb;
BEGIN
    -- Get token data
    SELECT jsonb_build_object(
        'address', t.address,
        'name', t.name,
        'symbol', t.symbol,
        'chain', t.chain,
		'market_cap', t.market_cap,
		'volume_24h', t.volume_24h,
		'liquidity', t.liquidity,
		'logo_uri', t.logo_uri
    )
    FROM social.tokens t
    WHERE t.address = NEW.token_address
    INTO token_data;

    -- Get user data
    SELECT jsonb_build_object(
        'id', u.id,
        'username', u.username,
        'telegram_id', u.telegram_id,
		'waitlisted', u.waitlisted
    )
    FROM public.user u
    WHERE u.id = NEW.user_id
    INTO user_data;

	-- Get group data
	SELECT jsonb_build_object(
		'id', g.id,
		'name', g.name,
		'logo_uri', g.logo_uri
	)
	FROM social.groups g
	WHERE g.id = NEW.group_id
	INTO group_data;

    -- Build the complete notification payload
    SELECT jsonb_build_object(
        'eventDate', CURRENT_TIMESTAMP,
		'groupName', group_data->>'name',
        'tokenPick', jsonb_build_object(
            'id', NEW.id,
            'token', token_data,
            'user', user_data,
            'group', group_data,
            'telegram_message_id', NEW.telegram_message_id,
            'price_at_call', NEW.price_at_call,
            'market_cap_at_call', NEW.market_cap_at_call,
            'supply_at_call', NEW.supply_at_call,
            'call_date', NEW.call_date,
            'highest_market_cap', NEW.highest_market_cap,
            'hit_date', NEW.hit_date
        )
    ) INTO pick_data;

    PERFORM pg_notify('social.token_picks', pick_data::text);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER token_pick_notify_trigger
    AFTER INSERT ON social.token_picks
    FOR EACH ROW
    EXECUTE FUNCTION social.notify_token_pick();

-- migrate:down
DROP TRIGGER IF EXISTS token_pick_notify_trigger ON social.token_picks;
DROP FUNCTION IF EXISTS social.notify_token_pick();
