.echo prompt
CREATE TABLE users
(
    id INTEGER PRIMARY KEY,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE
);

.echo off
.sleep 1
.echo prompt
INSERT INTO users (id, first_name, last_name, email)
VALUES (1, 'Alice', 'Johnson', 'alice@example.com'),
       (2, 'Bob', 'Smith', 'bob@example.com'),
       (3, 'Charlie', 'Brown', 'charlie@example.com'),
       (4, 'David', 'Lee', 'david@example.com');

.echo off
.sleep 1
.echo prompt
.rows

.echo off
.sleep 1
.echo prompt
.rows off

.echo off
.sleep 1
.echo prompt
SELECT
    id AS "Id",
    first_name AS "First Name",
    last_name AS "Last Name",
    email AS "Email"
FROM users;

.echo off
.sleep 1
.echo prompt
.rows on

.echo off
.sleep 1
.echo prompt
SELECT
    id AS "Id",
    first_name AS "First Name",
    last_name AS "Last Name",
    email AS "Email"
FROM users;
