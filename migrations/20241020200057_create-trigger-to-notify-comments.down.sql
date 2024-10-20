-- Add down migration script here
DROP TRIGGER IF EXISTS notify_new_comment_trigger ON social.comments;
DROP FUNCTION IF EXISTS notify_new_comment();
