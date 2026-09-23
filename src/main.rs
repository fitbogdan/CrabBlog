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


// use crate::common::decode_body_field;
use crate::handlers::{get_con, handle_login, handle_register, handle_reply, send_css, send_home, send_image};
use crate::http::{ContentType, get_cookie, get_multipart_part_bytes, send_404, send_response};
use crate::datatypes::{Attempts, Credentials};
use crate::post::send_post;




fn main(){
    run_server();
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


pub fn read_request_bytes(stream: &mut TcpStream, content_kind: &mut ContentType) -> Option<Vec<u8>>{
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
    let mut content_type: http::ContentType = http::ContentType::Nothing;
    
    for line in String::from_utf8_lossy(&data[0..header_end]).lines(){
        if line.to_lowercase().starts_with("content-length:"){
            if let Some((_, v)) = line.split_once(":"){
                content_length = match v.trim().parse::<usize>(){
                    Ok(v) => Some(v),
                    Err(_) => None,
                };

            }
        }

        if line.to_lowercase().starts_with("content-type:"){
            if let Some((_,v)) = line.split_once(":"){
                let v = v.trim();
                let (kind, params) = match v.split_once(";"){
                    Some((k,p)) => (k.trim(), p),
                    None => (v,"")
                };

                content_type = match kind{
                    "application/x-www-form-urlencoded" => ContentType::Form,
                    "multipart/form-data" => {
                        let mut boundary = String::new();
                        for p in params.split(";"){
                            if let Some(b) = p.trim().strip_prefix("boundary="){
                                boundary = b.to_string();
                            }
                        }

                        ContentType::Multipart(boundary)
                    },
                    _ => ContentType::Nothing,
                };
            }
        }
    }

    *content_kind = content_type;

    const MAX_BODY_BYTES: usize = 10 * 1024 * 1024; // 10 MB
    if content_length > Some(MAX_BODY_BYTES){
        return None
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
    let mut content_type: ContentType = ContentType::Form;
    let request_bytes = match read_request_bytes(&mut stream, &mut content_type){
        Some(r) => r,
        None => return,
    };
    
    let request = String::from_utf8_lossy(&request_bytes);


    let token = get_cookie(&request, "token");


    let (user_id, is_admin): (Option<u32>,bool) = match &token{
        Some(token) => {
            let conn = get_con(&db);


            match db_service::user_from_cookie(&conn, &token){
                Some((id, admin)) => (Some(id),admin),
                None => (None,false)
            }
        },
        _=> (None,false)
    };

    let credentials: Credentials = Credentials::new(user_id, is_admin);



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
        ("GET", []) => send_home(&mut stream, credentials, &db),
        // ("GET", ["static", "images", filename]) => send_image(&mut stream, &format!("static/images/{}",filename)),
        ("GET", ["post", id])  => {
            match id.parse::<u32>() {
                Ok(post_id) => send_post(&mut stream, post_id, &db, credentials),
                Err(_) => send_404(&mut stream),
            }
        },
        ("GET", ["style.css"]) => send_css(&mut stream, "static/style.css".to_string()),

        ("POST", ["comment", comment_id, "delete", post_id]) => {



            let comment_id = match comment_id.parse::<u32>(){
                Ok(id) => id,
                _ => {
                    send_404(&mut stream);
                    return;
                }
            };

            let post_id = match post_id.parse::<u32>(){
                Ok(id) => id,
                _ => {
                    send_404(&mut stream);
                    return;
                }
            };


            if credentials.is_admin == false{ //Check if the comment belongs to the user:
                let author_id = {
                    let conn = get_con(&db);
                    db_service::get_comment_author(&conn, comment_id)
                };


                let author_id = match author_id{
                    Some(id) => id,
                    None => {
                        send_404(&mut stream);
                        return;
                    }
                };

                if let Some(uid) = user_id{
                    if author_id != uid{
                        send_response(&mut stream, 401, "text/html", "You are not allowed to do that", None);
                        return;
                    }
                }
                else{
                    send_404(&mut stream);
                    return;
                }
            }


            {
                let conn = &get_con(&db);
                db_service::delete_comment(conn, comment_id);
            }

            send_response(&mut stream, 302, "text/html", "", Some(&format!("Location: /post/{}", post_id)));
            
        }

        ("GET", ["post", post_id, "image"]) => {

            let post_id = match post_id.parse::<u32>(){
                Ok(post_id) => post_id,
                _ => {
                    send_404(&mut stream);
                    return;
                },
            };
            let image_path = {
                let conn = get_con(&db);
                db_service::image_path_from_post_id(post_id, &conn) 
            };

            let image_path = match image_path{
                Some(p) => p,
                None => {
                    send_404(&mut stream);
                    return;
                }
            };


            send_image(&mut stream, &image_path);
        }

        ("POST", ["post", post_id, "reply"]) => {




            handle_reply(&mut stream, post_id, None, body,user_id, Utc::now(), &db);
        
        },



        ("POST", ["post", post_id, "reply", parent_id]) =>  {

            handle_reply(&mut stream, post_id, Some(parent_id), body, user_id, Utc::now(), &db);
        
        },

        ("POST", ["post", "create"]) => {
            if credentials.is_admin == false{
                send_response(&mut stream, 401, "text/html", "You are not allowed to do that", None);
                return;
            }

            // let headline = decode_body_field(body, "title");
            // let body = decode_body_field(body, "body");
            // let post = datatypes::Post { id: 0, title: headline, body, date: Utc::now() };
            // {
            //     let conn = &get_con(&db);
            //     db_service::send_post(conn, &post);
            // }

            // send_response(&mut stream, 302, "text/html", "", Some("Location: /"));

            if let ContentType::Multipart(boundary) = content_type{
                handlers::process_multipart_post(&mut stream, &request_bytes, &boundary, &db);
            }
            else{
                return
            }
        },

        ("POST", ["post", post_id, "delete"]) => {


            if credentials.is_admin == false{
                send_response(&mut stream, 401, "text/html", "You are not allowed to do that", None);
                return;
            }


            let post_id_fin = match post_id.parse::<u32>(){
                    Ok(n) => n,
                    Err(_) => {
                        send_response(&mut stream, 400, "text/html", "Error, post_id wrong", None);
                        return;
                    }
            };
            {

                let conn = get_con(&db);
                db_service::delete_post(&conn, post_id_fin);

            }


            send_response(&mut stream, 302, "text/html", "", Some("Location: /"));
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

        ("GET", ["write"]) => {

            if credentials.is_admin == false{
                send_response(&mut stream, 401, "text/html", "You are not allowed to do that", None);
                return;
            }


            let html = std::fs::read_to_string("static/create_post.html").unwrap();

            send_response(&mut stream, 200, "text/html", &html, None);

        },


        ("POST", ["upload"]) => {



            if credentials.is_admin == false{
                send_response(&mut stream, 401, "text/html", "You are not allowed to do that", None);
                return;
            }



            if let ContentType::Multipart(boundary) = content_type{
                    // http::parse_multipart(&boundary, &request_bytes, 0);
                let image_bytes = get_multipart_part_bytes(&boundary, &request_bytes,0);

                if let Some(image_bytes) = image_bytes{

                let ext = if image_bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
                    "png"
                } else if image_bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
                    "jpg"
                } else if image_bytes.starts_with(b"GIF8") {
                    "gif"
                } else {
                    send_response(& mut stream, 400, "text/html", "Not an image", None);
                    return;
                };


                    std::fs::write(format!("static/images/image.{}", ext), image_bytes).unwrap();
                }
                
            }

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