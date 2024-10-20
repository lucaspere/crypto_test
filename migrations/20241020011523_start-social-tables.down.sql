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