use std::net::{TcpListener, TcpStream};
use std::io::{Read,Write};
use std::thread;
pub mod post;
pub mod datatypes;
pub mod common;
use crate::datatypes::PostCard;
use crate::post::send_post;
use crate::common::items;

fn main(){
    run_server();
}

pub fn render_home_page(home_loc: &str, post_card_loc: &str, items: Vec<PostCard>) -> String{
    println!("Rendering home page, with {} posts", items.len());

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


pub fn read_request(request_line: String, path: &mut String, method: &mut String){

    let mut parts = request_line.split_whitespace();
    *method = parts.next().unwrap_or("").to_string();
    *path = parts.next().unwrap_or("").to_string();
}

pub fn handle_connection(mut stream: TcpStream){
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let request = String::from_utf8_lossy(&buffer[..]);

    println!("Got incoming request:\n{}", request);


    let mut method: String = "".to_string();
    let mut path: String = "".to_string();

    read_request(request.to_string(), &mut path, &mut method);

    println!("Got path: {}, and method: {}", path, method);

    match (method.as_str(), path.as_str()){
        ("GET", "/") => send_home(&mut stream),
        ("GET", p) if p.starts_with("/post/")  => {
            let id_str = p.strip_prefix("/post/").unwrap_or("");
            let id: u32 = id_str.parse().unwrap_or(0);

            send_post(&mut stream, id);
        },
        ("GET", "/style.css") => send_css(&mut stream, "static/style.css".to_string()),
        _ => {

            println!("Got 404: {}", path);
            let rb = "<h1> 404 Not found </h1>";
            let response = format!(
                "HTTP/1.1 404 NOT FOUND \r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
                rb.len(),
                rb,
            );

            stream.write_all(response.as_bytes()).unwrap();
        },
    }
}

pub fn run_server(){
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    println!("Listening on http://127.0.0.1:8080");

    let concurency = true;

    for stream in listener.incoming(){

        match stream {
            Ok(stream) => {
                if concurency == true {
                    thread::spawn(move ||{
                        handle_connection(stream);
                    });
                }
                else{
                    handle_connection(stream);
                }
            },
            Err(e) => eprintln!("Connection failed! {}", e)
        }

    }
}