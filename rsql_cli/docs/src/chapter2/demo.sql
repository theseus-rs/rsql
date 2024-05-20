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
.describe users

.echo off
.sleep 1
.echo prompt
.desc users
