use std::net::{TcpListener, TcpStream};
use std::io::{Read,Write};
use std::sync::{Arc, Mutex};
use std::thread;
pub mod post;
pub mod datatypes;
pub mod common;
pub mod db_service;
use chrono::{DateTime, Utc};
use rusqlite::Connection;

use crate::datatypes::{PostCard, Comment};
use crate::db_service::send_comment;
use crate::post::send_post;
use crate::common::{items, send_response};

fn main(){
    run_server();
}

pub fn render_home_page(home_loc: &str, post_card_loc: &str, items: Vec<PostCard>) -> String{
    // println!("Rendering home page, with {} posts", items.len());

    let mut final_post_html: String = "".to_string();

    for i in 0..items.len(){
        let mut post_html = std::fs::read_to_string(post_card_loc).unwrap();

        post_html = post_html.replace("{{POST_DATE}}", &items[i].date);
        post_html = post_html.replace("{{POST_TITLE}}", &items[i].title);
        post_html = post_html.replace("{{POST_DESCRIPTION}}", &items[i].description);
        post_html = post_html.replace("{{POST_IMAGE_URL}}", "/image");
        post_html = post_html.replace("{{POST_ID}}", &format!("/post/{}", &items[i].id));

        final_post_html = final_post_html + &post_html;
    }




    let mut rb = std::fs::read_to_string(home_loc).unwrap();
    rb = rb.replace("{{POSTS}}", &final_post_html);

    rb
}

pub fn send_home(stream: &mut TcpStream){

    let rb = render_home_page("static/home.html", "static/post_card.html", items());
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
        rb.len(),
        rb,
    );

    stream.write_all(response.as_bytes()).unwrap();
}




pub fn send_css(stream: &mut TcpStream, file_path: String){
    match std::fs::read_to_string(&file_path){
        Ok(contents) => {
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/css\r\nContent-Length: {}\r\n\r\n{}",
                contents.len(), contents
            );

            stream.write_all(response.as_bytes()).unwrap();
        },
        Err(_) => {
            let rb = "<h1>404 Not found</h1>";
            let response = format!(
                "HTTP/1.1 404 NOT FOUND\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
                rb.len(), rb
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
    }
}

pub fn send_image(stream: &mut TcpStream, file_path: &str){
    let bytes = match std::fs::read(file_path){
        Ok(b) => b,
        Err(_) => {send_404(stream); return;}
    };

    let content_type = if file_path.ends_with(".jpg") { "image/jpeg" } else { "application/octet-stream" };

    let headers = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
        content_type, bytes.len()
    );

    stream.write_all(headers.as_bytes()).unwrap();
    stream.write_all(&bytes).unwrap();

}


pub fn read_request(request_line: &str, path: &mut String, method: &mut String){

    let mut parts = request_line.split_whitespace();
    *method = parts.next().unwrap_or("").to_string();
    *path = parts.next().unwrap_or("").to_string();
}

pub fn get_body(request: &str) -> &str{
    request.split_once("\r\n\r\n")
           .map(|(_, body)| body)
           .unwrap_or("")
}

pub fn send_404(stream: &mut TcpStream){
    let rb = "<h1> 404 Not found </h1>";
    let response = format!(
        "HTTP/1.1 404 NOT FOUND \r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
        rb.len(),
        rb,
    );

    stream.write_all(response.as_bytes()).unwrap();
}

pub fn get_cookie(request: &str, cookie_name: &str) -> Option<String>{

    for line in request.lines(){
        if line.to_lowercase().starts_with("cookie:"){
            let rest = line[7..].trim();

            for pair in rest.split(";"){
                let pair_trimmed = pair.trim();
                if let Some((key, value)) = pair_trimmed.split_once("="){
                    if key == cookie_name{

                        return Some(value.to_string());
                    }
                }
            }

        }
    }

    None
}


pub fn handle_reply(stream: &mut TcpStream, post_id: &str, parent_id: Option<&str>, body: &str, user_id: u32, date: DateTime<Utc>, db: &Arc<Mutex<Connection>>){
    
    let post_id_fin = match post_id.parse::<u32>(){
        Ok(n) => n,
        Err(_) => {
            send_response(stream, 400, "text/html", "Error, post_id wrong");
            return;
        }
    };

    let parent_id_fin: Option<u32> = match parent_id{

        None => None,
        Some(v) => match v.parse::<u32>(){

            Ok(n) => Some(n),
            Err(_) => {send_response(stream, 400, "text/html", "Bad Parent Id"); return;}
        },
    };


    let comment = Comment { id: 0, post_id: post_id_fin, user_id: user_id, body: body.to_string(), parent_id: parent_id_fin, date: date};


    {
    let conn = db.lock().unwrap();
    send_comment(&conn, &comment);
    } //Scope conn so it releases the lock


    let response = format!("HTTP/1.1 302 Found\r\nLocation: /post/{}\r\n\r\n", post_id);
    stream.write_all(response.as_bytes()).unwrap();

}

pub fn handle_connection(mut stream: TcpStream, db: Arc<Mutex<Connection>>){
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let request = String::from_utf8_lossy(&buffer[..]);


    let user_id: u32 = match get_cookie(&request, "user_id"){
        Some(v) => v.parse().unwrap_or(0),
        None => 0,
    };

    
    println!("Got this user: {}", user_id);

    println!("Got incoming request:\n{}", request);


    let mut method: String = "".to_string();
    let mut path: String = "".to_string();
    let body = get_body(&request);

    read_request(&request.to_string(), &mut path, &mut method);
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    // println!("Got path: {}, and method: {}", path, method);




    match (method.as_str(), segments.as_slice()){
        ("GET", []) => send_home(&mut stream),
        ("GET", ["image"]) => send_image(&mut stream, "static/zeth.jpg"),
        ("GET", ["post", id])  => {
            match id.parse::<u32>() {
                Ok(post_id) => send_post(&mut stream, post_id, db),
                Err(_) => send_404(&mut stream),
            }
        },
        ("GET", ["style.css"]) => send_css(&mut stream, "static/style.css".to_string()),

        ("POST", ["post", post_id, "reply"]) => handle_reply(&mut stream, post_id, None, body, user_id, Utc::now(), &db),

        ("POST", ["post", post_id, "reply", parent_id]) =>  handle_reply(&mut stream, post_id, Some(parent_id), body, user_id, Utc::now(), &db),
        _ => {
            send_404(&mut stream);
        },
    }
}

pub fn run_server(){
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    println!("Listening on http://127.0.0.1:8080");

    let concurrency = true;
    let conn = db_service::create_db();
    let db = Arc::new(Mutex::new(conn));

    for stream in listener.incoming(){

        match stream {
            Ok(stream) => {
                if concurrency == true {
                    let db = Arc::clone(&db);

                    thread::spawn(move ||{
                        handle_connection(stream,db);
                    });
                }
                else{
                    handle_connection(stream,Arc::clone(&db));
                }
            },
            Err(e) => eprintln!("Connection failed! {}", e)
        }

    }
}