use std::net::TcpStream;
use std::time::Instant;
use crate::datatypes::RegisterResult::{ServerError, Success, UsernameTaken};
use crate::http::{self, send_404, send_response};
use crate::db_service::{generate_token, get_post_cards, send_comment};
use crate::datatypes::{Attempts, Comment, Credentials, PostCard};
use crate::common::{self, auth_bar_html};
use chrono::{DateTime, Utc};
use std::io::{Write};
use std::sync::{Arc, Mutex, MutexGuard};
use rusqlite::Connection;
use crate::common::{decode_body_field};
use crate::db_service;
use crate::datatypes;




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



pub fn render_home_page(home_loc: &str, items: Vec<PostCard>, credentials: Credentials) -> String{
    // println!("Rendering home page, with {} posts", items.len());


    let auth_html = auth_bar_html(credentials.user_id);

    let mut final_post_html: String = "".to_string();

    // print!("\n\n\n\n{}\n\n\n\n", credentials.is_admin);
    let delete_button = match credentials.is_admin{
        true => std::fs::read_to_string(common::DELETE_POST_BUTTON).unwrap(),
        false => "".to_string(),
    };

    for i in 0..items.len(){
        let mut post_html = std::fs::read_to_string(common::POST_CARD).unwrap();
        let date = DateTime::parse_from_rfc3339(&items[i].date).unwrap().with_timezone(&Utc);

        post_html = post_html.replace("{{POST_DATE}}", &common::get_readable_date(date));
        post_html = post_html.replace("{{POST_TITLE}}", &items[i].title);
        post_html = post_html.replace("{{POST_DESCRIPTION}}", &items[i].description);
        post_html = post_html.replace("{{POST_IMAGE_URL}}", &format!("/post/{}/image", &items[i].id));
        post_html = post_html.replace("{{ADMIN_DELETE}}", &delete_button);
        post_html = post_html.replace("{{POST_ID}}", &format!("{}", &items[i].id));
        post_html = post_html.replace("{{REPLIES_COUNT}}", &format!("{}", &items[i].comment_count));
        
        final_post_html = final_post_html + &post_html;
    }




    let mut rb = std::fs::read_to_string(home_loc).unwrap();
    rb = rb.replace("{{POSTS}}", &final_post_html);
    rb = rb.replace("{{AUTH_BUTTONS}}", auth_html);
    rb = rb.replace("{{USER_TYPE}}", if credentials.is_admin == true { "ADMIN" } else {"REGULAR USER/GUEST"});
    rb = rb.replace("{{ADMIN_PANEL}}", if credentials.is_admin == true {  r#"<a href="write">Add New Post</a>"#  } else {""});
    rb
}

pub fn send_home(stream: &mut TcpStream, credentials: Credentials, db: &Arc<Mutex<Connection>>){

    let post_cards ={
        let conn = get_con(db);
        get_post_cards(&conn)
    };




    let rb = render_home_page("static/home.html", post_cards, credentials);
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

    let ext = match file_path.rsplit_once('.'){
        Some((_,e)) => e,
        None => ""
    };

    let content_type = match ext{
        "jpg" => "image/jpeg",
        "png" => "image/png",
        "gif"  => "image/gif",
        _ => "application/octet-stream",
    };


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

    let register_html = std::fs::read_to_string("static/register.html").unwrap();

    let username = decode_body_field(body, "username");
    let password = decode_body_field(body, "password");
    let password2 = decode_body_field(body, "password_again");


    if password != password2{
        let register_html = register_html.replace("{{ERROR}}", "The passwords don't match!");

        send_response(stream, 400, "text/html", &register_html, None);

        return
    }

    if password.len() < 8{
        let register_html = register_html.replace("{{ERROR}}", "The password is too short!");

        send_response(stream, 400, "text/html", &register_html, None);

        return
    }

    if password.len() > 70{
        let register_html = register_html.replace("{{ERROR}}", "The password is too long!");

        send_response(stream, 400, "text/html", &register_html, None);

        return
    }

    if password.starts_with(" ") || password.ends_with(" "){
        let register_html = register_html.replace("{{ERROR}}", "The password cannot start or end with spaces. (Check if you wrote it correctly)");

        send_response(stream, 400, "text/html", &register_html, None);

        return
    }

    if username.len() > 20{
        let register_html = register_html.replace("{{ERROR}}", "Your username is too long");

        send_response(stream, 400, "text/html", &register_html, None);

        return
    }
    else if username.len() < 3{
        let register_html = register_html.replace("{{ERROR}}", "Your username is too short");

        send_response(stream, 400, "text/html", &register_html, None);

        return
    }
    
    for c in username.chars(){
        //If its not an alphanumeric char or a _, bad request
        if !(c.is_ascii_alphabetic() || c == '_'){
            let register_html = register_html.replace("{{ERROR}}", "Make sure your username only contains letters, numbers, or \"_\"");

            send_response(stream, 400, "text/html", &register_html, None);

            return
        }
    }        




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

pub fn get_image_extension(image_bytes: &[u8]) -> Option<String>{
    let ext = if image_bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
        "png"
    } else if image_bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "jpg"
    } else if image_bytes.starts_with(b"GIF8") {
        "gif"
    } else {
        // send_response(& mut stream, 400, "text/html", "Not an image", None);
        return None;
    };


    return Some(ext.to_string());
}

pub fn process_multipart_post(stream: &mut TcpStream, body: &[u8], boundary: &str, db: &Arc<Mutex<Connection>>){
    let image_bytes = match http::get_multipart_part_bytes(boundary, body, 0){
        Some(b) => b,
        None => return,
    };

    let headline = match http::get_multipart_part_bytes(boundary, body, 1){
        Some(s) => String::from_utf8_lossy(&s).into_owned(),
        None => return
    };

    let post_body = match http::get_multipart_part_bytes(boundary, body, 2){
        Some(s) => String::from_utf8_lossy(&s).into_owned(),
        None => return
    };

    let token = generate_token();
    let ext = get_image_extension(&image_bytes);
    if let Some(ext) = ext{

        let filename = format!("{}.{}",token,ext);
        let path = format!("static/images/{}", filename);
        std::fs::write(&path, image_bytes).unwrap();


        let post = datatypes::Post { id: 0, title: headline, body: post_body, date: Utc::now(), image_path: Some(path)};
        {
            let conn = &get_con(&db);
            db_service::send_post(&conn, &post);
        }

        send_response(stream, 302, "text/html", "", Some("Location: /"));
    }
    else{
        send_response(stream, 400, "text/html", "Not an image", None);
        return;
    }



}