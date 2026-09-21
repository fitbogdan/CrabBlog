use rusqlite::{Connection,params};
use crate::datatypes::RegisterResult::{ServerError, UsernameTaken};
use crate::{datatypes::Comment};
use chrono::{DateTime, Duration, Utc};
use crate::common::{decode_body,encode_html};
use crate::datatypes::{Post, PostCard, RegisterResult};



pub const SESSION_DAYS: i64 = 30;
pub const SESSION_SECONDS: u64 = SESSION_DAYS as u64 * 86400;


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
                date_joined TEXT NOT NULL,
                is_admin BOOLEAN not NULL DEFAULT FALSE
            );
            CREATE TABLE IF NOT EXISTS sessions(
                token TEXT NOT NULL PRIMARY KEY,
                user_id INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                expires_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS posts(
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                body TEXT NOT NULL,
                date TEXT NOT NULL
            );" 
    ).unwrap();

    conn
}

pub fn get_comment(conn: &Connection, post_id: u32) -> Vec<Comment>{
    //Returns escaped comment
    //Example: "<" will be returned as &lt; Etc..

    let mut stmt = conn.prepare(
        "SELECT c.id, c.post_id, c.parent_id, c.user_id, c.body, c.date, u.username 
        FROM comments c
        JOIN users u ON u.id = c.user_id
        WHERE c.post_id = ?1"
    ).unwrap();

    let rows = stmt.query_map([post_id], |row|{
        let date_str: String = row.get(5)?;
        let date = DateTime::parse_from_rfc3339(&date_str).unwrap().with_timezone(&Utc);
        let body_raw: String = row.get(4)?;


        let body_encoded = encode_html(&body_raw);
        let username_raw: String = row.get(6).unwrap();

        // let body_decoded = decode_body(&body_raw);

        Ok(Comment{
            id: row.get(0)?,
            post_id: row.get(1)?,
            parent_id: row.get(2)?,
            user_id: row.get(3)?,
            body: body_encoded,
            date: date,
            username: Some(encode_html(&username_raw)),
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


pub fn send_post(conn: &Connection, post: &Post){

    conn.execute(
        "INSERT INTO posts (title, body, date) VALUES (?1, ?2, ?3)",
        params![
            post.title,
            post.body,
            Utc::now().to_rfc3339()
        ]
    ).unwrap();
}


//Basically redundant, not really used:
pub fn get_posts(conn: &Connection) -> Vec<Post>{


    let mut stmt = conn.prepare(
        "SELECT id, title, body, date FROM posts"
    ).unwrap();


    let rows = stmt.query_map([], |row| {


        let date_str: String = row.get(3).unwrap();
        let date = DateTime::parse_from_rfc3339(&date_str).unwrap().with_timezone(&Utc);

        Ok(
            Post{
                id: row.get(0).unwrap(),
                title: row.get(1).unwrap(),
                body: row.get(2).unwrap(),
                date: date
            }
        )
    }).unwrap();

    rows.map(|r| r.unwrap()).collect()
}

pub fn get_post_cards(conn: &Connection) -> Vec<PostCard>{

    let mut stmt = conn.prepare(
        "SELECT id,title,substr(body, 1, 100),date FROM posts ORDER BY date DESC"
    ).unwrap();

    let rows = stmt.query_map([], |row|{

        let date_str: String = row.get(3).unwrap();
        // let date = DateTime::parse_from_rfc3339(&date_str).unwrap().with_timezone(&Utc);

        let body: String = row.get(2).unwrap();

        let first_line = match body.lines().next(){
            Some(line) => line.to_string(),
            _ => String::new()
        };


        Ok(
            PostCard{
                id: row.get(0).unwrap(),
                title: row.get(1).unwrap(),
                description: first_line,
                date: date_str,
                image_url: "TODO".to_string(),
            }
        )
    }).unwrap();

    rows.map(|r| r.unwrap()).collect()
}

pub fn delete_post(conn: &Connection, id: u32){
    conn.execute(
        "DELETE FROM comments WHERE post_id = ?1", params![id]
    ).unwrap();

    conn.execute(
        "DELETE FROM posts WHERE id = ?1", params![id]
    ).unwrap();

}

pub fn get_post(conn: &Connection, id: u32) -> Option<Post>{

    
    let mut stmt = conn.prepare(
        "SELECT title, body, date FROM posts WHERE id = ?1"
    ).unwrap();

    let row = stmt.query_one([id], |r| {
        let date_str: String = r.get(2).unwrap();
        let date = DateTime::parse_from_rfc3339(&date_str).unwrap().with_timezone(&Utc);
        Ok(
            Post{
                id: id,
                title: r.get(0).unwrap(),
                body: r.get(1).unwrap(),
                date: date
            }
        )
    });



    //RETURN the value wrapped in option
    match row{
        Ok(r) => Some(r),
        _ => None
    }

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

    //Let it silently execute. If it fails, no panic since its not crucial to logging in.
    let _ = conn.execute(
        "DELETE FROM sessions WHERE expires_at < ?1",
        params![Utc::now().to_rfc3339()] 
    );
    

    /*
        TODO!!!!!!!!!!

        BUG: User can spam create new tokens on every login.

        We will rate limit each IP, but we will also have to cap 5 sessions PER user.
    
     */


    //We have a match
    let cookie = generate_cookie(conn, user_id);

    return cookie;
}

pub fn create_user(conn: &Connection, username: &str, password: &str) -> RegisterResult{
    //1: Check if already exists
        //If yes => Redirect

    //2: Hash password
    //3: Push to db
    //4: Hand new token

    let res = bcrypt::hash(password, 12);
    let password_hash = match res {
        Ok(t) => t,
        Err(_) => return ServerError
    };



    let res = conn.execute(
    "INSERT INTO users (username, password, date_joined)
        VALUES(?1, ?2, ?3)",
        
        params![username, password_hash, Utc::now().to_rfc3339()] 
    );
    match res{
        Ok(_) => {}, //Was able to insert
        Err(_) =>{
            return UsernameTaken
        }  // Duplicate
    }


    let user_id = conn.last_insert_rowid() as u32;

    let result = match generate_cookie(conn, user_id){
        Some(r) => RegisterResult::Success(r),
        None => RegisterResult::ServerError
    };

    result
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
        "INSERT INTO sessions (user_id,token,created_at,expires_at)
        VALUES(?1, ?2, ?3, ?4)", 
        params![user_id, token, Utc::now().to_rfc3339(), (Utc::now()+Duration::days(SESSION_DAYS)).to_rfc3339()]
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

pub fn user_from_cookie(conn: &Connection, cookie: &str) -> Option<(u32,bool)> {
    let mut stmt = conn.prepare(
        "SELECT sessions.user_id, sessions.expires_at, users.is_admin 
        FROM sessions 
        JOIN users ON users.id = sessions.user_id
        WHERE sessions.token = ?1 AND sessions.expires_at > ?2"
    ).unwrap();

    

    let result = stmt.query_one(params![cookie, Utc::now().to_rfc3339()], |r|{
        let user_id: u32 = r.get(0).unwrap();
        let is_admin: bool = r.get(2).unwrap();
        Ok((user_id, is_admin))
    });


    match result{
        Ok(i) => Some(i),
        _ => None,
    }
}

pub fn logout_user(conn: &Connection, token: &str){
    conn.execute(
        "DELETE FROM sessions WHERE token = ?1",
        params![token]
    ).unwrap();
}



