-- Add up migration script here
-- Create a function to notify about new comments
CREATE OR REPLACE FUNCTION social.notify_new_comment() RETURNS TRIGGER AS $$
DECLARE
    group_id_val bigint;
BEGIN
    -- Get the group_id from the token_call table
    SELECT group_id INTO group_id_val
    FROM social.token_calls
    WHERE id = NEW.token_call_id;

    -- Notify about the new comment with the group_id
    PERFORM pg_notify(
        'new_comment',
        json_build_object(
            'user_id', NEW.user_id,
            'comment_id', NEW.id,
            'group_id', group_id_val
        )::text
    );

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create a trigger to call the notify function when a new comment is added
CREATE TRIGGER notify_new_comment_trigger
AFTER INSERT ON social.comments
FOR EACH ROW
EXECUTE FUNCTION social.notify_new_comment();
