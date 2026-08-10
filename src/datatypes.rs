pub struct PostCard{
    pub title: String,
    pub description: String,
    pub date: String,
}

impl PostCard{
    pub fn new(title: String, description: String, date: String) -> PostCard{
        PostCard{
            title,
            description,
            date,
        }
    }
}