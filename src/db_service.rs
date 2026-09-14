use rusqlite::{Connection,params};
use crate::datatypes::Comment;
use chrono::{DateTime, Utc};
use crate::common::{decode_body,encode_html};

pub fn create_db() -> Connection{


    let conn = Connection::open("blog.db").unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS comments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            post_id INTEGER NOT NULL,
            parent_id INTEGER,
            user_id INTEGER NOT NULL,
            body TEXT NOT NULL,
            date TEXT NOT NULL
    )", 
        []
    ).unwrap();

    conn
}

pub fn get_comment(conn: &Connection, post_id: u32) -> Vec<Comment>{

    let mut stmt = conn.prepare(
        "SELECT id, post_id, parent_id, user_id, body, date FROM comments WHERE post_id = ?1"
    ).unwrap();

    let rows = stmt.query_map([post_id], |row|{
        let date_str: String = row.get(5).unwrap();
        let date = DateTime::parse_from_rfc3339(&date_str).unwrap().with_timezone(&Utc);
        let body_raw: String = row.get(4).unwrap();


        let body_encoded = encode_html(&body_raw);

        // let body_decoded = decode_body(&body_raw);

        Ok(Comment{
            id: row.get(0).unwrap(),
            post_id: row.get(1).unwrap(),
            parent_id: row.get(2).unwrap(),
            user_id: row.get(3).unwrap(),
            body: body_encoded,
            date: date,
        })
    }).unwrap();


    rows.map(|r| r.unwrap()).collect()
}


pub fn send_comment(conn: &Connection, comment: &Comment){

    let body_decoded = decode_body(&comment.body);
    
    conn.execute(
        "INSERT INTO comments (post_id, parent_id, user_id, body, date) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            comment.post_id,
            comment.parent_id,
            comment.user_id,
            body_decoded,
            comment.date.to_rfc3339(),
        ]
    ).unwrap();
}


