use crate::datatypes::PostCard;
use crate::common::{items,comments,send_response};
use std::net::{TcpStream};
use std::io::{Write};

pub fn render_comments(post_id: u32) -> String{
    let comments = comments();

    let comment_template = std::fs::read_to_string("static/comment.html").unwrap();
    let mut comment_body = String::new();
    for i in comments{
        if i.post_id != post_id{
            continue;
        }
        let cur = 
            comment_template.replace("{{USERNAME}}", &format!("{} - Zeth", i.user_id))
            .replace("{{DATE_POSTED}}", &i.date.to_string())
            .replace("{{COMMENT}}", &i.body)
            .replace("{{POST_ID}}", &i.post_id.to_string())
            .replace("{{COMMENT_ID}}", &i.id.to_string());

        comment_body.push_str(&cur);
    } 

    comment_body

}


pub fn get_post_text(id: u32)-> String{
    format!("Zeth's Post Text - {}", id)
}


pub fn send_post(stream: &mut TcpStream, id: u32){
    let items = items();

    let mut post: Option<&PostCard> = None;
    for i in items.iter(){
        if i.id == id {
            post = Some(i);
            break;
        }
    } 

    let mut rb = std::fs::read_to_string("static/post.html").unwrap();


    rb = match post {
        Some(p) => {
            

            let comments = render_comments(p.id);
            

            rb.replace("{{POST_TITLE}}", &p.title)
            .replace("{{POST_ID}}", &p.id.to_string())
            .replace("{{POST_DATE}}", &p.date)
            .replace("{{POST_TEXT}}", &get_post_text(id))
            .replace("{{COMMENTS}}", &comments)
        },
        None => "<h1> 404 Not found </h1>".to_string(),
    };


    send_response(stream, 200, "text/html", &rb);
}