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

pub fn log_in(username: &str, password: &str, conn: &Connection) -> Option<String>{

    let mut stmt = conn.prepare(
        "SELECT password, id FROM users WHERE username = ?1"
    ).unwrap();


    let result = stmt.query_one([username], |row| {
        let id: u32 = row.get(1).unwrap();
        let hash: String = row.get(0).unwrap();        
        Ok((id , hash))
    });

    let (user_id, stored_password_hash) = match result{
        Ok(r) => r,
        _ => return None, 
    };


    let is_match = match bcrypt::verify(password, &stored_password_hash){
        Ok(true) => true,
        _ => false,
    };

    if !is_match{
        return None;
    }


    //We have a match
    let cookie = generate_cookie(conn, user_id);

    return cookie;
}

pub fn create_user(conn: &Connection, username: &str, password: &str) -> Option<String>{
    //1: Check if already exists
        //If yes => Redirect

    //2: Hash password
    //3: Push to db
    //4: Hand new token

    let res = bcrypt::hash(password, 12);
    let password_hash = match res {
        Ok(t) => t,
        Err(_) => return None
    };



    let res = conn.execute(
    "INSERT INTO users (username, password, date_joined)
        VALUES(?1, ?2, ?3)",
        
        params![username, password_hash, Utc::now().to_rfc3339()] 
    );
    match res{
        Ok(_) => {}, //Was able to insert
        Err(_) => return None // Duplicate
    }


    let user_id = conn.last_insert_rowid() as u32;

    generate_cookie(conn, user_id)
}


pub fn generate_token() -> String{

    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).unwrap();


    let mut token = String::new();

    for b in bytes{

        //{:02x} -> Padded by 0, lowercase
        token.push_str(&format!("{:02x}", b));
    }

    token
}

pub fn generate_cookie(conn: &Connection, user_id: u32) -> Option<String>{

    let token = generate_token();

    let result = conn.execute(
        "INSERT INTO sessions (user_id,token,created_at)
        VALUES(?1, ?2, ?3)", 
        params![user_id, token, Utc::now().to_rfc3339()]
    );

    match result{
       Ok(_) => Some(token),
       _ => None,
    }
}

pub fn cookie_from_user(conn: &Connection, user_id: u32) -> Option<String>{
    //Get user_id,
    //Query sessions table for cookie

    let mut stmt = conn.prepare(
        "SELECT token FROM sessions WHERE user_id = ?1"
    ).unwrap();

    let result = stmt.query_one([user_id], |r|{
        let token: String = r.get(0).unwrap();
        Ok(token)
    });

    let cookie = match result{
        Ok(c) => Some(c),
        _ => None,
    };

    cookie 
}

pub fn user_from_cookie(conn: &Connection, cookie: &str) -> Option<u32>{
    let mut stmt = conn.prepare(
        "SELECT user_id FROM sessions WHERE token = ?1"
    ).unwrap();

    let result = stmt.query_one([cookie], |r|{
        let user_id: u32 = r.get(0).unwrap();
        Ok(user_id)
    });


    let id = match result{
        Ok(i) => Some(i),
        _ => None,
    };

    id
}

pub fn logout_user(conn: &Connection, token: &str){
    conn.execute(
        "DELETE FROM sessions WHERE token = ?1",
        params![token]
    ).unwrap();
}



