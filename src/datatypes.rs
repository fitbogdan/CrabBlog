use chrono::{DateTime, Utc};


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

pub struct Comment{
    pub id: u32,
    pub post_id: u32,
    pub user_id: u32,
    pub date: DateTime<Utc>,
    pub body: String,
    pub parent_id: Option<u32>,
}

impl Comment{
    pub fn new(
        id: u32,
        post_id: u32,
        user_id:u32,
        date: DateTime<Utc>,
        body: String,
        parent_id: Option<u32>,
    ) -> Comment{
        Comment { id, post_id, user_id, date, body, parent_id }
    }
}