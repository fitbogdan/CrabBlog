use crate::datatypes::{PostCard,Comment};
use chrono::{Utc,Duration};
use std::net::{TcpStream};
use std::io::{Write};

pub fn send_response(stream: &mut TcpStream, status: u32, content_type: &str, body: &str){


    let reason = match status {
        200 => "OK",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown",
    };

    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
        status, reason, content_type, body.len(), body
    );

    stream.write_all(response.as_bytes()).unwrap();
}

pub fn items() -> Vec<PostCard> {
    vec![
    PostCard::new(
    "About Zeth".to_string(),
    "Zeth is the most important resource, arguably, in the world".to_string(),
    "19th August 2020".to_string(),
    "a".to_string(),
    1,
    ),
    PostCard::new(
    "About Poop".to_string(),
    "Poop is the most important resource, arguably, in the world".to_string(),
    "19th August 2020".to_string(),
    "a".to_string(),
    2,
    ),
    PostCard::new(
    "About Zeth's Poop".to_string(),
    "Zeth's Poop is the most important resource, in the whole universe".to_string(),
    "19th August 2020".to_string(),
    "a".to_string(),
    3,
    )
    ]
}

pub fn comments() -> Vec<Comment>{
    vec![
        Comment::new(
            1,
            1,
            1,
            Utc::now(),
            "Zethen Machen".to_string(),
            None,
        ),

        Comment::new(
            1,
            1,
            1,
            Utc::now()+Duration::minutes(30),
            "Zethen Machen 30 minuten Hunden Wursten".to_string(),
            None,
        ),
    ]
}

pub fn get_username(id: u32) -> &'static str{
    match id {
        0 => "Bogdan",
        1 => "Zeth",
        2 => "Cat",
        3 => "Cats are losers",
        _ => "Weird Id",
    }
}