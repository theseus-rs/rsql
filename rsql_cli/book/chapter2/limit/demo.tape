Type "CREATE TABLE users (id INTEGER, email TEXT);"
Enter
Sleep 2s

Type "INSERT INTO users (id, email) VALUES (1, 'alice@example.com'), (2, 'bob@example.com');"
Enter
Sleep 2s

Type ".limit"
Enter
Sleep 2s

Type ".limit 1"
Enter
Sleep 2s

Type "SELECT id, email FROM users;"
Enter
Sleep 2s

Type ".limit 0"
Enter
Sleep 2s

Type "SELECT id, email FROM users;"
