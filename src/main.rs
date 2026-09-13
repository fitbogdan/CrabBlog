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
use crate::common::{comments, items, send_response};

fn main(){
    run_server();
}

pub fn render_home_page(home_loc: &str, post_card_loc: &str, items: Vec<PostCard>, comments: Vec<Comment>) -> String{
    // println!("Rendering home page, with {} posts", items.len());

    let mut final_post_html: String = "".to_string();

    for i in 0..items.len(){
        let mut post_html = std::fs::read_to_string(post_card_loc).unwrap();

        post_html = post_html.replace("{{POST_DATE}}", &items[i].date);
        post_html = post_html.replace("{{POST_TITLE}}", &items[i].title);
        post_html = post_html.replace("{{POST_DESCRIPTION}}", &items[i].description);
        post_html = post_html.replace("{{POST_IMAGE_URL}}", "https://upload.wikimedia.org/wikipedia/commons/5/57/German_shepard_female.jpg");
        post_html = post_html.replace("{{POST_ID}}", &format!("/post/{}", &items[i].id));

        final_post_html = final_post_html + &post_html;
    }




    let mut rb = std::fs::read_to_string(home_loc).unwrap();
    rb = rb.replace("{{POSTS}}", &final_post_html);

    rb
}

pub fn send_home(stream: &mut TcpStream){

    let rb = render_home_page("static/home.html", "static/post_card.html", items(), comments());
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


pub fn read_request(request_line: String, path: &mut String, method: &mut String){

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

pub fn handle_connection(mut stream: TcpStream, db: Arc<Mutex<Connection>>){
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let request = String::from_utf8_lossy(&buffer[..]);

    println!("Got incoming request:\n{}", request);


    let mut method: String = "".to_string();
    let mut path: String = "".to_string();
    let body = get_body(&request);

    read_request(request.to_string(), &mut path, &mut method);
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    // println!("Got path: {}, and method: {}", path, method);

    match (method.as_str(), segments.as_slice()){
        ("GET", []) => send_home(&mut stream),
        ("GET", ["post", id])  => {
            match id.parse::<u32>() {
                Ok(post_id) => send_post(&mut stream, post_id, db),
                Err(_) => send_404(&mut stream),
            }
        },
        ("GET", ["style.css"]) => send_css(&mut stream, "static/style.css".to_string()),

        ("POST", ["post", id, "reply"]) => {
            
            let post_id = match id.parse::<u32>(){
                Ok(n) => n,
                Err(_) => {
                    send_response(&mut stream, 400, "text/html", "Error, post_id wrong");
                    return;
                }
            };
            let comment = Comment { id: 0, post_id: post_id, user_id: 0, body: body.to_string(), parent_id: None, date: Utc::now()};

            {
            let conn = db.lock().unwrap();
            send_comment(&conn, &comment);
            } //Scope conn so it releases the lock



            let response = format!("HTTP/1.1 302 Found\r\nLocation: /post/{}\r\n\r\n", post_id);
            stream.write_all(response.as_bytes()).unwrap();
        },

        ("POST", ["post", id, "reply", comment_parent_id]) => {

            //TODO match Id with OK



            let post_id = match id.parse::<u32>(){
                Ok(n) => n,
                Err(_) => {
                    send_response(&mut stream, 400, "text/html", "Error, post_id wrong");
                    return;
                }
            };


            let parent_id = match comment_parent_id.parse::<u32>(){
                Ok(n) => n,
                Err(_) => { 
                    
                    send_response(&mut stream, 400, "text/html", "Error, parent comment id wrong"); 

                    return;
                }
            };

            let comment = Comment{id: 0, post_id: post_id, user_id: 0, body: body.to_string(), parent_id: Some(parent_id), date: Utc::now()};


            {

                let conn = db.lock().unwrap();
                send_comment(&conn, &comment);

            }


            let response = format!("HTTP/1.1 302 Found\r\nLocation: /post/{}\r\n\r\n", post_id);
            stream.write_all(response.as_bytes()).unwrap();
        }
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