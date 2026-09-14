use crate::datatypes::{PostCard,Comment};
use chrono::{Utc,Duration};
use std::net::{TcpStream};
use std::io::{Write};

pub fn decode(body: &str) -> String{

    let bytes = body.as_bytes();
    let mut result_bytes: Vec<u8> = Vec::new();
    let mut i = 0;
    while i < bytes.len(){

        if bytes[i] == b'%'{
            if i+2 < bytes.len() {

                let hex = std::str::from_utf8(&bytes[i+1..i+3]).unwrap();

                let byte = u8::from_str_radix(hex, 16).unwrap();

                result_bytes.push(byte);

                
                i = i+3;
            }
            else{
                i+=1;
            }
        }
        else if bytes[i] == b'+' {
            result_bytes.push(b' ');
            i+=1;
        }

        else{
            result_bytes.push(bytes[i]);
            i+=1;
        }
    }

    
    String::from_utf8(result_bytes).unwrap()
}

pub fn decode_body(body: &str) -> String{
    let mut comment_body = String::new();

    //Spliting the body:
    for pair in body.split('&'){
        match pair.split_once('='){
            Some(("body", value)) => {
                comment_body = decode(value) ;
            },
            _ => {}
        }
    }


    comment_body

}

pub fn encode_html(body: &str) -> String{
    let mut out = String::new();

    for c in body.chars(){
        match c {
            '<' => {
                out.push_str("&lt;");
            },
            '>' => {
                out.push_str("&gt;");
            }
            '&' => {
                out.push_str("&amp;")
            }
            '\'' => {
                out.push_str("&#39;");
            }

            '"' => {
                out.push_str("&quot;");
            }
            _ => {
                out.push(c);
            }
        }
    }


    out
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
            4,
            1,
            2,
            Utc::now(),
            "Personally I hate Zeth".to_string(),
            Some(1),
        ),

        Comment::new(
            5,
            1,
            2,
            Utc::now(),
            "You big fat piece of shit how dare you say that about Zeth i hope you get hemmoroids".to_string(),
            Some(1),
        ),

        Comment::new(
            2,
            1,
            1,
            Utc::now()+Duration::minutes(30),
            "Zethen Machen 30 minuten Hunden Wursten".to_string(),
            None,
        ),


        Comment::new(
            3,
            2,
            2,
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