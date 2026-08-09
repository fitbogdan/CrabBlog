use std::net::TcpListener;
use std::io::{Read,Write};

pub fn run_server(){
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    println!("Listening on http://127.0.0.1:8080");

    for stream in listener.incoming(){
        let mut stream = stream.unwrap();
        //unwrap -> If there is an error it crashes and prints the error


        let mut buffer = [0; 1024];
        stream.read(&mut buffer).unwrap();
        let request = String::from_utf8_lossy(&buffer[..]);

        println!("Got incoming request: {}", request);


        let items = vec!["About Niggers", "Why Poop Niggers are Alive", "White Niggers Really do exist"];
        let li_html: String = items.iter().map(|item| format!("<li><a href=\"https://www.akc.org/dog-breeds/german-shepherd-dog/\">{}</a></li>", item)).collect();


        
        let mut rb = std::fs::read_to_string("static/home.html").unwrap();
        rb = rb.replace("{{POSTS}}", &li_html);
        

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nConent-Length: {}\r\n\r\n{}",
            rb.len(),
            rb,
        );

        stream.write_all(response.as_bytes()).unwrap();

    }
}