use rusqlite::Connection;

use crate::datatypes::{Comment, Credentials, Post};
use crate::common::{self, auth_bar_html};
use crate::http::{send_response};
use crate::db_service::{get_comment, get_post};
use crate::handlers::{get_con};
use std::net::{TcpStream};
use std::collections::{HashMap};
use std::sync::{Arc, Mutex};


pub fn render_comments(post_id: u32, db: &Arc<Mutex<Connection>>) -> (String, usize){
    let comments = {
        let conn = get_con(db);
        get_comment(&conn, post_id)
    };

    if comments.is_empty() {
        return ("".to_string(), 0);
    }

    let comments_length: usize = comments.len();

    let comment_template = std::fs::read_to_string(common::COMMENT_HTML).unwrap();
    let subcomment_template = std::fs::read_to_string(common::SUBCOMMENT_HTML).unwrap();
    let mut comment_body = String::new();

    let mut subcomments: HashMap<u32, Vec<Comment>> = HashMap::new();
    let mut parents: Vec<Comment> = Vec::new();




    for i in comments{
        if i.post_id != post_id{
            continue;
        }

        match i.parent_id{
            Some(parent) => subcomments.entry(parent).or_default().push(i),
            None => parents.push(i),
        }
    }


    for i in parents{

        let username = match &i.username{
            Some(i) => i,
            _ => "",
        };


        let mut cur = 
            comment_template.replace("{{USERNAME}}", &username)
            .replace("{{DATE_POSTED}}", &i.date.to_string())
            .replace("{{COMMENT}}", &i.body)
            .replace("{{POST_ID}}", &i.post_id.to_string())
            .replace("{{COMMENT_ID}}", &i.id.to_string());
        let mut cur_subcomments = String::new();

        if let Some(subs) = subcomments.get(&i.id){
            for s in subs{
                let username = match &s.username{
                    Some(u) => u,
                    _ => "",
                };

                let cur_subcomment = 
                    subcomment_template.replace("{{USERNAME}}", username)
                                       .replace("{{DATE}}", &s.date.to_string())
                                       .replace("{{BODY}}", &s.body.to_string());


                cur_subcomments.push_str(&cur_subcomment);
            }

            cur = cur.replace("{{SUB_COMMENTS}}", &cur_subcomments);
        }
        else{
            cur = cur.replace("{{SUB_COMMENTS}}", "");
        }


        comment_body.push_str(&cur);
    } 

    (comment_body, comments_length)

}


pub fn get_post_text(id: u32)-> String{
    format!("Zeth's Post Text - {}", id)
}


pub fn send_post(stream: &mut TcpStream, id: u32, db: &Arc<Mutex<Connection>>, credentials: Credentials){

    let post: Option<Post> = {
        let conn = get_con(db);
        get_post(&conn, id)
    };

    let mut rb = std::fs::read_to_string("static/post.html").unwrap();


    let auth_html = auth_bar_html(credentials.user_id);
    let comment_box = match credentials.user_id{
        Some(_) => std::fs::read_to_string(common::AUTH_COMMENT_BOX).unwrap(),
        None => std::fs::read_to_string(common::GUEST_COMMENT_BOX).unwrap(),
    };

    rb = match post {
        Some(p) => {
            

            let (comments_html, comment_count) = render_comments(p.id,db);
            

            rb.replace("{{POST_TITLE}}", &p.title)
            .replace("{{COMMENT_BOX}}", &comment_box)
            .replace("{{NR_COMMENTS}}", &comment_count.to_string())
            .replace("{{POST_ID}}", &p.id.to_string())
            .replace("{{POST_DATE}}", &p.date.to_string())
            .replace("{{POST_TEXT}}", &p.body)
            .replace("{{COMMENTS}}", &comments_html)
            .replace("{{AUTH_BUTTONS}}", auth_html)
        },
        None => "<h1> 404 Not found </h1>".to_string(),
    };


    send_response(stream, 200, "text/html", &rb, None);
}