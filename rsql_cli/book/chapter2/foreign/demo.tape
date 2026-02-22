Type "CREATE TABLE users (id INTEGER PRIMARY KEY, email TEXT NOT NULL UNIQUE);"
Enter
Sleep 2s

Type "CREATE TABLE orders (id INTEGER PRIMARY KEY, user_id INTEGER REFERENCES users(id));"
Enter
Sleep 2s

Type ".foreign"
