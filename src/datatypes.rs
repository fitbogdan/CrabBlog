use std::{collections::HashMap, sync::{Arc, Mutex}, time::Instant};

use chrono::{DateTime, Utc};

pub type Attempts = Arc<Mutex<HashMap<String, Vec<Instant>>>>;

pub enum RegisterResult{
    Success(String), // Send the cookie
    UsernameTaken,
    ServerError
}
pub struct PostCard{
    pub title: String,
    pub description: String,
    pub date: String,
    pub image_url: String,
    pub id: u32,
}

impl PostCard{
    pub fn new(title: String, 
        description: String, 
        date: String,
        image_url: String, 
        id: u32) -> PostCard{
            PostCard{
                title,
                description,
                date,
                image_url,
                id,
            }
    }
}

pub struct Post{
    pub id: u32,
    pub title: String,
    pub body: String,
    pub date: DateTime<Utc>,
}

impl Post{
    pub fn new(id: u32, title: String, body: String, date: DateTime<Utc>) -> Post{
        Post { id, title, body, date }
    }
}
pub struct Comment{
    pub id: u32,
    pub post_id: u32,
    pub user_id: u32,
    pub username: Option<String>,
    pub date: DateTime<Utc>,
    pub body: String,
    pub parent_id: Option<u32>,
}

impl Comment{
    pub fn new(
        id: u32,
        post_id: u32,
        user_id:u32,
        username: Option<String>,
        date: DateTime<Utc>,
        body: String,
        parent_id: Option<u32>,
    ) -> Comment{
        Comment { id, username, post_id, user_id, date, body, parent_id }
    }
}

pub struct User{
    pub id: u32,
    pub username: String,
    pub password_hash: String,
    pub date_joined: DateTime<Utc>,
}

impl User{
    pub fn new(
        id: u32,
        username: String,
        password_hash: String,
        date_joined: DateTime<Utc>,
    ) -> User{
        User { id, username, password_hash, date_joined }
    }
}

pub struct Session{
    pub token: String,
    pub user_id: u32,
    pub created_at: DateTime<Utc>,
}


impl Session{
    pub fn new(token: String, user_id: u32, created_at: DateTime<Utc>) -> Session{
        Session { token, user_id, created_at }
    }
}

pub struct Credentials{
    pub user_id: Option<u32>,
    pub is_admin: bool
}

impl Credentials{
    pub fn new(user_id: Option<u32>, is_admin: bool) -> Credentials{
        Credentials { user_id, is_admin }
    }
}
