use std::collections::HashMap;
// use std::hash::Hash;
use std::net::{TcpListener, TcpStream};
use std::io::{Read};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
pub mod post;
pub mod datatypes;
pub mod common;
pub mod db_service;
pub mod handlers;
pub mod http;
use chrono::{Utc};
use rusqlite::Connection;


use crate::handlers::{get_con, handle_login, handle_register, handle_reply, send_css, send_home, send_image};
use crate::http::{get_cookie, send_404,send_response};
use crate::datatypes::{Attempts, PostCard};
use crate::post::send_post;
use crate::common::{auth_bar_html};




fn main(){
    run_server();
}

pub fn render_home_page(home_loc: &str, items: Vec<PostCard>, user_id: Option<u32>) -> String{
    // println!("Rendering home page, with {} posts", items.len());

    let auth_html = auth_bar_html(user_id);

    let mut final_post_html: String = "".to_string();

    for i in 0..items.len(){
        let mut post_html = std::fs::read_to_string(common::POST_CARD).unwrap();

        post_html = post_html.replace("{{POST_DATE}}", &items[i].date);
        post_html = post_html.replace("{{POST_TITLE}}", &items[i].title);
        post_html = post_html.replace("{{POST_DESCRIPTION}}", &items[i].description);
        post_html = post_html.replace("{{POST_IMAGE_URL}}", "/image");
        post_html = post_html.replace("{{POST_ID}}", &format!("/post/{}", &items[i].id));
        
        final_post_html = final_post_html + &post_html;
    }




    let mut rb = std::fs::read_to_string(home_loc).unwrap();
    rb = rb.replace("{{POSTS}}", &final_post_html);
    rb = rb.replace("{{AUTH_BUTTONS}}", auth_html);
    rb
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


pub fn read_request_bytes(stream: &mut TcpStream) -> Option<Vec<u8>>{
    let mut buffer:[u8; 1024] = [0; 1024];

    let mut n = match stream.read(&mut buffer){
        Ok(n)  => n,
        Err(_) => return None,
    };
    let mut data: Vec<u8> = Vec::new();
    let mut header_end = 0;
    while n > 0{
        data.extend_from_slice(&buffer[0..n]);

        if let Some(index) = data.windows(4).position(|w| w == b"\r\n\r\n"){
            header_end = index;
            break;
        }

        n = match stream.read(&mut buffer){
            Ok(n)  => n,
            Err(_) => return None,
        };
    }


    if n == 0 { 
        return None; 
    }

    //Get Content-Length:
    let mut content_length: Option<usize> = None;

    for line in String::from_utf8_lossy(&data[0..header_end]).lines(){
        if line.to_lowercase().starts_with("content-length: "){
            if let Some((_, v)) = line.split_once(":"){
                content_length = match v.trim().parse::<usize>(){
                    Ok(v) => Some(v),
                    Err(_) => None,
                };


                break; 
            }
        }
    }

    if let Some(l) = content_length{
        while data.len() < l + header_end + 4{
            n = match stream.read(&mut buffer){
                Ok(n)  => n,
                Err(_) => return None,
            };

            if n == 0{
                break;
            }

            data.extend_from_slice(&buffer[0..n]);
        }
    }



    Some(data) 
}

pub fn handle_connection(mut stream: TcpStream, db: Arc<Mutex<Connection>>, attempts: Attempts){

    let request_bytes = match read_request_bytes(&mut stream){
        Some(r) => r,
        None => return,
    };

    let request = String::from_utf8_lossy(&request_bytes);


    let token = get_cookie(&request, "token");


    let user_id: Option<u32> = match &token{
        Some(token) => {
            let conn = get_con(&db);


            db_service::user_from_cookie(&conn, &token)
        },

        _=> None,
    };


    println!("Got this user: {:?}", user_id);

    println!("Got incoming request:\n{}", request);


    let mut method: String = "".to_string();
    let mut path: String = "".to_string();
    let body = get_body(&request);

    read_request(&request.to_string(), &mut path, &mut method);
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    // println!("Got path: {}, and method: {}", path, method);

    let cookie_duration = db_service::SESSION_SECONDS;


    match (method.as_str(), segments.as_slice()){
        ("GET", []) => send_home(&mut stream, user_id),
        ("GET", ["image"]) => send_image(&mut stream, "static/zeth.jpg"),
        ("GET", ["post", id])  => {
            match id.parse::<u32>() {
                Ok(post_id) => send_post(&mut stream, post_id, &db, user_id),
                Err(_) => send_404(&mut stream),
            }
        },
        ("GET", ["style.css"]) => send_css(&mut stream, "static/style.css".to_string()),

        ("POST", ["post", post_id, "reply"]) => {




            handle_reply(&mut stream, post_id, None, body,user_id, Utc::now(), &db);
        
        },



        ("POST", ["post", post_id, "reply", parent_id]) =>  {

            handle_reply(&mut stream, post_id, Some(parent_id), body, user_id, Utc::now(), &db);
        
        }
        
        ("POST", ["login"]) => {
            handle_login(&mut stream, body, &db, cookie_duration, &attempts);
        },
        ("POST", ["register"]) => {
            handle_register(&mut stream, body, &db, cookie_duration, &attempts);            
        },

        ("GET", ["login"]) => {
            // Render login page
            let mut login_html = std::fs::read_to_string("static/login.html").unwrap();
            login_html = login_html.replace("{{ERROR}}", "");
            send_response(&mut stream, 200, "text/html", &login_html, None);

        },

        ("GET", ["register"]) => {
            let mut register_html = std::fs::read_to_string("static/register.html").unwrap();
            register_html = register_html.replace("{{ERROR}}", "");
            send_response(&mut stream, 200, "text/html", &register_html, None);

        },

        ("POST", ["logout"]) => {
            let conn = get_con(&db);
            if let Some(t) = token{
                db_service::logout_user(&conn, &t);
            }
            send_response(&mut stream, 302, "text/html", "", 
                Some("Set-Cookie: token=; Max-Age=0; Path=/; HttpOnly\r\nLocation: /")
            );
        },

        // ("GET", ["boom"]) =>{
        //     let con = get_con(&db);
        //     // let con = db.lock().unwrap();
        //     panic!("Bubuie!");
        // }

        
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
    let attempts: Arc<Mutex<HashMap<String, Vec<Instant>>>> = Arc::new(Mutex::new(HashMap::new()));

    for stream in listener.incoming(){

        match stream {
            Ok(stream) => {
                stream.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
                if concurrency == true {
                    let db = Arc::clone(&db);
                    let attempts = Arc::clone(&attempts);
                    thread::spawn(move ||{
                        handle_connection(stream,db,attempts);
                    });
                }
                else{
                    handle_connection(stream,Arc::clone(&db), Arc::clone(&attempts));
                }
            },
            Err(e) => eprintln!("Connection failed! {}", e)
        }

    }
}