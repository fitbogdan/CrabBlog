use std::{io::Write, net::TcpStream};




pub fn send_response(stream: &mut TcpStream, status: u32, content_type: &str, body: &str, extra_header: Option<&str>){


    let reason = match status {
        200 => "OK",
        302 => "Found",
        400 => "Bad Request" ,
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown",
    };

    let extra = match extra_header{
        Some(h) => format!("{}\r\n", h),
        None => String::new(),
    };


    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n{}\r\n{}",
        status, reason, content_type, body.len(), extra, body
    );

    stream.write_all(response.as_bytes()).unwrap();
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