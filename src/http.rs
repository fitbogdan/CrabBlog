use std::{io::Write, net::TcpStream};

pub enum ContentType{
    Nothing,
    Form,
    Multipart(String), //Boundary
    Other(String)
}


pub fn get_multipart_part_bytes<'a>(boundary: &str, body: &'a[u8], from_part: u32) -> Option<&'a[u8]>{

    let separator = format!("--{}", boundary);
    let separator = separator.as_bytes();

    let mut i = 0;
    let mut parts = 0;
    let mut start = None;

    /*
    
        How this request body looks like (Roughly):

        --Boundary(A random string which isn't in the body, with -- before it)
        Headers
        \r\n\r\n
        Part 1

        --Boundary
        Headers
        \r\n\r\n
        Part 2

        --Boundary

        ... etc ...
        \r\n--Boundary-- (This is the end)
    
    */


    //Loop over form_part boundaries, meaning we get to the form_part-th part of the request.
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


    //Skipping over the headers:
    while i+4 <= body.len(){
        if &body[i..i+4] == b"\r\n\r\n"{
            data_start = Some(i+4);
            break; 
        }

        i+=1
    } 
    //Found the start of the data or not:
    let data_start = match data_start {
        Some(i) => i,
        None => return None
    };


    let mut data_end = None;


    //Data ends when we meet the next marker, which is \r\n--boundary, \r\n because its just a new line
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




    // println!("{}, THIS IS THE DATA::  !!!  {:?}", start, String::from_utf8_lossy(&body[data_start..data_end]));

    //Return the slice of the body bytes which corresponds to the form_part-th part of the request.
    Some(&body[data_start..data_end])
}

pub fn get_multipart_parts<'a>(boundary: &str, body: &'a[u8]) -> Vec<(String, &'a[u8])>{


    let separator = format!("--{}", boundary);
    let separator = separator.as_bytes();

    let end_marker = format!("\r\n--{}", boundary);
    let end_marker = end_marker.as_bytes();

    let mut res: Vec<(String, &[u8])> = Vec::new();

    let end_request_marker = format!("--{}--", boundary);
    let end_request_marker = end_request_marker.as_bytes();




    let mut i = 0;

    while i+end_request_marker.len() <= body.len(){
        if &body[i..i+end_request_marker.len()] == end_request_marker{
            break;
        }


        if &body[i..i+separator.len()] == separator{
            //First part
            i+=2;

            //i is now the start of the header
            let mut j = i;
            let mut header_end = None;
            while j+4 <= body.len(){
                if &body[j..j+4] == b"\r\n\r\n"{
                    header_end = Some(j);

                    break;
                    //Get the name of the field:
                }
                j+=1
            }



            let header_end = match header_end{
                Some(p) => p,
                None => return Vec::new()
            };

            let headers = String::from_utf8_lossy(&body[i..header_end]);
            let mut name = String::new();
            for line in headers.lines(){
                if let Some((header, values)) = line.split_once(":"){
                    if header.trim().to_lowercase() == "content-disposition"{
                        for p in values.split(";"){
                            let p = p.trim();

                            if let Some((field, value)) = p.split_once("="){
                                if field == "name"{
                                    name = value.trim_matches('"').to_string();
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            // println!("Got NAMES FROM THIS MULTIPART WHOOO:        {}", &name);


            //i is the start of the headers,
            //j is the end of the headers.
            let data_start = header_end+4;
            let mut k = data_start;
            let mut data_end = None;
            while k+end_marker.len() <= body.len(){
                if &body[k..k+end_marker.len()] == end_marker{
                    data_end = Some(k);
                    break;
                }
                k+=1;
            }

            let data_end = match data_end{
                Some(k) => k,
                None => return Vec::new()
            };

            // println!("{} {}", data_start, data_end);

            let bytes = &body[data_start..data_end];


            res.push((name, bytes));
        }
        i+=1
    }

    // println!("The result: {:?}", res);

    // println!("The first: {} \n\n\n\n\n The second: {}", String::from_utf8_lossy(res[0].1), String::from_utf8_lossy(res[1].1));

    return res;
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