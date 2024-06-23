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
.format

.echo off
.sleep 1
.echo prompt
.format ascii

.echo off
.sleep 1
.echo prompt
SELECT * FROM users;

.echo off
.sleep 1
.echo prompt
.format csv

.echo off
.sleep 1
.echo prompt
SELECT * FROM users;

.echo off
.sleep 1
.echo prompt
.format expanded

.echo off
.sleep 1
.echo prompt
SELECT * FROM users;

.echo off
.sleep 1
.echo prompt
.format html

.echo off
.sleep 1
.echo prompt
SELECT * FROM users;

.echo off
.sleep 1
.echo prompt
.format json

.echo off
.sleep 1
.echo prompt
SELECT * FROM users;

.echo off
.sleep 1
.echo prompt
.format jsonl

.echo off
.sleep 1
.echo prompt
SELECT * FROM users;

.echo off
.sleep 1
.echo prompt
.format markdown

.echo off
.sleep 1
.echo prompt
SELECT * FROM users;

.echo off
.sleep 1
.echo prompt
.format plain

.echo off
.sleep 1
.echo prompt
SELECT * FROM users;

.echo off
.sleep 1
.echo prompt
.format psql

.echo off
.sleep 1
.echo prompt
SELECT * FROM users;

.echo off
.sleep 1
.echo prompt
.format sqlite

.echo off
.sleep 1
.echo prompt
SELECT * FROM users;

.echo off
.sleep 1
.echo prompt
.format tsv

.echo off
.sleep 1
.echo prompt
SELECT * FROM users;

.echo off
.sleep 1
.echo prompt
.format unicode

.echo off
.sleep 1
.echo prompt
SELECT * FROM users;

.echo off
.sleep 1
.echo prompt
.format xml

.echo off
.sleep 1
.echo prompt
SELECT * FROM users;

.echo off
.sleep 1
.echo prompt
.format yaml

.echo off
.sleep 1
.echo prompt
SELECT * FROM users;
