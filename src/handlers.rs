use std::collections::HashMap;
use std::net::TcpStream;
use std::time::Instant;
use crate::datatypes::RegisterResult::{ServerError, Success, UsernameTaken};
use crate::render_home_page;
use crate::http::{send_response,send_404};
use crate::db_service::{send_comment};
use crate::datatypes::{Comment, Attempts};
use crate::common::{items};
use chrono::{DateTime, Utc};
use std::io::{Write};
use std::sync::{Arc, Mutex, MutexGuard};
use rusqlite::Connection;
use crate::common::{decode_body_field};
use crate::db_service;




pub fn get_con(db: &Arc<Mutex<Connection>>) -> MutexGuard<'_, Connection>{
    let con = match db.lock(){
        Ok(c) => c,
        Err(poisoned) => poisoned.into_inner()
    };

    con
}

pub fn handle_reply(stream: &mut TcpStream, post_id: &str, parent_id: Option<&str>, body: &str, user_id: Option<u32>, date: DateTime<Utc>, db: &Arc<Mutex<Connection>>){
    
    
    let uid = match user_id{
        Some(id) => id,
        None => {
            send_response(stream, 401, "text/html", "<h1>Log in to comment</h1>", None);
            return;
        }
    };


    let post_id_fin = match post_id.parse::<u32>(){
        Ok(n) => n,
        Err(_) => {
            send_response(stream, 400, "text/html", "Error, post_id wrong", None);
            return;
        }
    };

    let parent_id_fin: Option<u32> = match parent_id{

        None => None,
        Some(v) => match v.parse::<u32>(){

            Ok(n) => Some(n),
            Err(_) => {send_response(stream, 400, "text/html", "Bad Parent Id", None); return;}
        },
    };


    let comment = Comment { id: 0, post_id: post_id_fin, user_id: uid, body: body.to_string(), parent_id: parent_id_fin, date: date, username: None};


    {
        let conn = get_con(&db);
        send_comment(&conn, &comment);
    } //Scope conn so it releases the lock


    let response = format!("HTTP/1.1 302 Found\r\nLocation: /post/{}\r\n\r\n", post_id);
    stream.write_all(response.as_bytes()).unwrap();

}


pub fn send_home(stream: &mut TcpStream, user_id: Option<u32>){


    let rb = render_home_page("static/home.html", "static/post_card.html", items(), user_id);
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

pub fn handle_rate_limiting(stream: &mut TcpStream, attempts: &Attempts, ip_prefix: &str,
                            rate_limit_window_secs: u64, rate_max_limit_attempts: usize) -> bool{
    
    

    let ip = match stream.peer_addr(){
        Ok(i) => {
            // print!("PEER ADDR !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!! -> \n{}\n\n\n\n",i);
            format!("{}:{}", ip_prefix,i.ip().to_string())
        },
        Err(_) => return false
    };


    let mut map = match attempts.lock(){
        Ok(l) => l,
        _ => return false
    };

    let list = map.entry(ip).or_default();



    list.retain(|t| t.elapsed().as_secs() < rate_limit_window_secs);

    if list.len() >= rate_max_limit_attempts{
        return false
    }

    list.push(Instant::now());

    return true

}

pub fn handle_login(stream: &mut TcpStream, body: &str, db: &Arc<Mutex<Connection>>, cookie_duration: u64, attempts: &Attempts){

    const RATE_LIMIT_WINDOW_SECS: u64 = 900; //15 mins
    const RATE_MAX_LIMIT_ATTEMPTS: usize = 5;

    if !handle_rate_limiting(stream, attempts, "login", RATE_LIMIT_WINDOW_SECS, RATE_MAX_LIMIT_ATTEMPTS){

        send_response(stream, 429, "text/html", "<h1>You are logging in too much</h1>", None);

        return
    }



    let username = decode_body_field(body, "username");
    let password = decode_body_field(body, "password");

    let cookie = {

        let conn = get_con(db);
        db_service::log_in(&username, &password, &conn)

    }; //Scope conn so it releases the lock

    match cookie{
        Some(c) => send_response(stream, 302, "text/html", "", Some(&format!("Set-Cookie: token={}; Max-Age={}; Path=/; HttpOnly\r\nLocation: /", c, cookie_duration))),
        None => {


            let mut login_html = std::fs::read_to_string("static/login.html").unwrap();
            login_html = login_html.replace("{{ERROR}}", "*Error! Wrong Password/Username*");
            send_response(stream, 401, "text/html", &login_html, None);
            // send_response(stream, 401, "text/html", "<h1>Log in failed! Check password/username.</h1>", None)
        }
    };
}



pub fn handle_register(stream: &mut TcpStream, body: &str, db: &Arc<Mutex<Connection>>, cookie_duration: u64, attempts: &Attempts){


    const RATE_LIMIT_WINDOW_SECS: u64 = 3600; //15 mins
    const RATE_MAX_LIMIT_ATTEMPTS: usize = 3;

    if !handle_rate_limiting(stream, attempts, "register", RATE_LIMIT_WINDOW_SECS, RATE_MAX_LIMIT_ATTEMPTS){
        send_response(stream, 429, "text/html", "<h1>You are making too many accounts</h1>", None);
        return;
    }


    let username = decode_body_field(body, "username");
    let password = decode_body_field(body, "password");

    let result =  {
        let conn = get_con(&db);
        db_service::create_user(&conn, &username, &password)
    };



    match result{
        Success(s) => send_response(stream, 302, "text/html", "", Some(&format!("Set-Cookie: token={}; Max-Age={}; Path=/; HttpOnly\r\nLocation: /", s, cookie_duration))),
        UsernameTaken => {

            let mut register_html = std::fs::read_to_string("static/register.html").unwrap();
            register_html = register_html.replace("{{ERROR}}", "*Username is Taken!*");

            send_response(stream, 409, "text/html", &register_html, None);
        }

        ServerError => {
            let mut register_html = std::fs::read_to_string("static/register.html").unwrap();
            register_html = register_html.replace("{{ERROR}}", "*Error: Server Issues*");

            send_response(stream, 500, "text/html", &register_html, None);
        }
    }

}