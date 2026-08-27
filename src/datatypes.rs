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