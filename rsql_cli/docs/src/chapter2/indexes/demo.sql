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
.indexes

.echo off
.sleep 1
.echo prompt
.indexes foo

.echo off
.sleep 1
.echo prompt
.indexes users
