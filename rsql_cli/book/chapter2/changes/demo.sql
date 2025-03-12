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
       (2, 'Bob', 'Smith', 'bob@example.com');

.echo off
.sleep 1
.echo prompt
.changes

.echo off
.sleep 1
.echo prompt
.changes off

.echo off
.sleep 1
.echo prompt
INSERT INTO users (id, first_name, last_name, email)
VALUES (3, 'Charlie', 'Brown', 'charlie@example.com');

.echo off
.sleep 1
.echo prompt
.changes on

.echo off
.sleep 1
.echo prompt
INSERT INTO users (id, first_name, last_name, email)
VALUES (4, 'David', 'Lee', 'david@example.com');
