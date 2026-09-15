use rusqlite::{Connection,params};
use crate::datatypes::Comment;
use chrono::{DateTime, Utc};
use crate::common::{decode_body,encode_html};

pub fn create_db() -> Connection{


    let conn = Connection::open("blog.db").unwrap();

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS comments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            post_id INTEGER NOT NULL,
            parent_id INTEGER,
            user_id INTEGER NOT NULL,
            body TEXT NOT NULL,
            date TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS users(
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                password TEXT NOT NULL,
                date_joined TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS sessions(
                token TEXT NOT NULL PRIMARY KEY,
                user_id INTEGER NOT NULL,
                created_at TEXT NOT NULL
            );" 
    ).unwrap();

    conn
}

pub fn get_comment(conn: &Connection, post_id: u32) -> Vec<Comment>{
    //Returns escaped comment
    //Example: "<" will be returned as &lt; Etc..

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

pub fn log_in(){
    //1: Check if user exists
        //No => Redirect

    //2: Verify agaisnt Hash password
    //3: Pull password from db
    //4: Check validity
    //If valid: Return Success!
    //5: Get user_id
    //6: Hand new token with cookie_from_user()

}

pub fn create_user(){
    //1: Check if already exists
        //If yes => Redirect

    //2: Hash password
    //3: Push to db
    //4: Hand new token

}

pub fn cookie_from_user(){
    //Get user_id,
    //Query sessions table for cookie
}

pub fn create_session(){
    //Generate cookie, push to sessions table
}


