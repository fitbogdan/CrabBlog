pub struct PostCard{
    pub title: String,
    pub description: String,
    pub date: String,
    pub image_url: String,
}

impl PostCard{
    pub fn new(title: String, description: String, date: String, image_url: String) -> PostCard{
        PostCard{
            title,
            description,
            date,
            image_url,
        }
    }
}