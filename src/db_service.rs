use rusqlite::Connection;
use crate::datatypes::Comment;


fn create_db() -> Connection{


    let conn = Connection::open("blog.db").unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS comments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            post_id INTEGER NOT NULL,
            parent_id INTEGER,
            user_id INTEGER NOT NULL,
            body TEXT NOT NULL,
            date TEXT NOT NULL
        ", 
        []
    ).unwrap();

    conn
}

