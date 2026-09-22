use std::{io::Write, net::TcpStream};

pub enum ContentType{
    Nothing,
    Form,
    Multipart(String), //Boundary
    Other(String)
}


pub fn parse_multipart<'a>(boundary: &str, body: &'a[u8], from_part: u32) -> Option<&'a[u8]>{

    let separator = format!("--{}", boundary);
    let separator = separator.as_bytes();

    let mut i = 0;
    let mut parts = 0;
    let mut start = None;

    while i+separator.len() <= body.len(){
        if &body[i..i+separator.len()] == separator{
            if parts == from_part{
                start = Some(i);
                break;
            }
            else{
                parts+=1;

            }
        }
        i+=1;
    }

    let start = match start{
        Some(s) => s,
        None => return None
    };


    let mut i = start + separator.len();
    let end_marker = format!("\r\n--{}", boundary);
    let end_marker = end_marker.as_bytes();
    let mut data_start = None;


    while i+4 <= body.len(){
        if &body[i..i+4] == b"\r\n\r\n"{
            data_start = Some(i+4);
            break; 
        }

        i+=1
    } 

    let data_start = match data_start {
        Some(i) => i,
        None => return None
    };


    let mut data_end = None;

    i = data_start;
    while i+end_marker.len() <= body.len(){
        if &body[i..i+end_marker.len()] == end_marker{
            data_end = Some(i);
            break;
        }
        i+=1;
    }


    let data_end = match data_end{
        Some(d) => d,
        None => return None
    };




    println!("{}, THIS IS THE DATA::  !!!  {:?}", start, String::from_utf8_lossy(&body[data_start..data_end]));

    Some(&body[data_start..data_end])
}

pub fn send_response(stream: &mut TcpStream, status: u32, content_type: &str, body: &str, extra_header: Option<&str>){


    let reason = match status {
        200 => "OK",
        302 => "Found",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        409 => "Conflict",
        429 => "Too Many Requests",
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