Type "CREATE TABLE users (id INTEGER, email TEXT);"
Enter
Sleep 2s

Type "INSERT INTO users (id, email) VALUES (1, 'alice@example.com'), (2, 'bob@example.com');"
Enter
Sleep 2s

Type "SELECT * FROM users;"
Enter
Sleep 2s

Type ".format csv"
Enter
Sleep 1s
Type "SELECT * FROM users;"
Enter
Sleep 2s

Type ".format json"
Enter
Sleep 1s
Type "SELECT * FROM users;"
