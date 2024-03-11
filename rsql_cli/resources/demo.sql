CREATE TABLE contacts
(
    contact_id INTEGER PRIMARY KEY,
    first_name TEXT NOT NULL,
    last_name  TEXT NOT NULL,
    email      TEXT NOT NULL UNIQUE,
    phone      TEXT NOT NULL UNIQUE
);

INSERT INTO contacts (first_name, last_name, email, phone)
VALUES ('Alice', 'Johnson', 'alice@example.com', '555-1234'),
       ('Bob', 'Smith', 'bob@example.com', '555-5678'),
       ('Charlie', 'Brown', 'charlie@example.com', '555-9876'),
       ('David', 'Lee', 'david@example.com', '555-4321');

SELECT *
FROM contacts;
