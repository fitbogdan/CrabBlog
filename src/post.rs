use crate::datatypes::PostCard;
use crate::items;
use std::net::{TcpStream};
use std::io::{Write};

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
            rb.replace("{{POST_TITLE}}", &p.title)
            .replace("{{POST_ID}}", &p.id.to_string())
            .replace("{{POST_DATE}}", &p.date)
            .replace("{{POST_TEXT}}", &get_post_text(id))
        },
        None => "<h1> 404 Not found </h1>".to_string(),
    };

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
        rb.len(), rb
    );

    stream.write_all(response.as_bytes()).unwrap();
}