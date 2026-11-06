Type "CREATE TABLE users (id INTEGER, email TEXT);"
Enter
Sleep 2s

Type "INSERT INTO users (id, email) VALUES (1, 'alice@example.com'), (2, 'bob@example.com');"
Enter
Sleep 2s

Type ".changes"
Enter
Sleep 2s

Type ".changes off"
Enter
Sleep 2s

Type "INSERT INTO users (id, email) VALUES (3, 'charlie@example.com');"
Enter
Sleep 2s

Type ".changes on"
Enter
Sleep 2s

Type "INSERT INTO users (id, email) VALUES (4, 'david@example.com');"
