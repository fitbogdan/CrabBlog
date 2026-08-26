use std::net::{TcpListener, TcpStream};
use std::io::{Read,Write};

pub mod datatypes;
use crate::datatypes::PostCard;

pub fn render_home_page(home_loc: &str, post_card_loc: &str, items: Vec<PostCard>) -> String{
    println!("Rendering home page, with {} posts", items.len());

    let mut final_post_html: String = "".to_string();

    for i in 0..items.len(){
        let mut post_html = std::fs::read_to_string(post_card_loc).unwrap();

        post_html = post_html.replace("{{POST_DATE}}", &items[i].date);
        post_html = post_html.replace("{{POST_TITLE}}", &items[i].title);
        post_html = post_html.replace("{{POST_DESCRIPTION}}", &items[i].description);
        post_html = post_html.replace("{{POST_IMAGE_URL}}", "https://upload.wikimedia.org/wikipedia/commons/5/57/German_shepard_female.jpg");

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



pub fn get_post_id(){

}

pub fn send_test(stream: &mut TcpStream){
    let rb = "<h1>HELLO</h1>";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
        rb.len(),
        rb,
    );

    stream.write_all(response.as_bytes()).unwrap();
}

fn items() -> Vec<PostCard> {
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

pub fn read_request(request_line: String, path: &mut String, method: &mut String){

    let mut parts = request_line.split_whitespace();
    *method = parts.next().unwrap_or("").to_string();
    *path = parts.next().unwrap_or("").to_string();
}

pub fn run_server(){
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    println!("Listening on http://127.0.0.1:8080");

    for stream in listener.incoming(){
        let mut stream = stream.unwrap();
        //unwrap -> If there is an error it crashes and prints the error


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
            ("GET", "/test") => send_test(&mut stream),
            _ => println!("Error"),
        }



    }
}