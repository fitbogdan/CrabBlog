use std::net::TcpStream;
use crate::render_home_page;
use crate::http::{send_response,send_404};
use crate::db_service::{send_comment};
use crate::datatypes::{Comment};
use crate::common::{items};
use chrono::{DateTime, Utc};
use std::io::{Write};
use std::sync::{Arc, Mutex};
use rusqlite::Connection;

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


    let comment = Comment { id: 0, post_id: post_id_fin, user_id: uid, body: body.to_string(), parent_id: parent_id_fin, date: date};


    {
    let conn = db.lock().unwrap();
    send_comment(&conn, &comment);
    } //Scope conn so it releases the lock


    let response = format!("HTTP/1.1 302 Found\r\nLocation: /post/{}\r\n\r\n", post_id);
    stream.write_all(response.as_bytes()).unwrap();

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