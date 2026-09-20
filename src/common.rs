use crate::datatypes::{PostCard};

pub const AUTH_COMMENT_BOX: &str = "static/components/auth_comment_box.html";
pub const GUEST_COMMENT_BOX: &str = "static/components/guest_comment_box.html";
pub const COMMENT_HTML: &str = "static/components/comment.html";
pub const POST_CARD: &str = "static/components/post_card.html";
pub const SUBCOMMENT_HTML: &str = "static/components/subcomment.html";




pub fn auth_bar_html(user_id: Option<u32>) -> &'static str{
    match user_id{
            Some(_) => {
                r#"
                    <form method="post" action="/logout">
                        <button type="submit" class="font-display text-lg text-ink truncate hover:text-green-400 hover:underline bg-transparent border-none p-0 cursor-pointer">
                            Logout
                        </button>
                    </form>
                "#
            },
            _ => {

                r#"
                    <a href="/login" class="font-display  text-lg text-ink truncate hover:text-green-400 hover:underline">
                        Log-In 
                    </a>


                    <p class="font-display text-lg text-ink truncate opacity-50">
                    / 
                    </p>

                    <a href="/register" class="font-display  text-lg text-ink truncate hover:text-green-400 hover:underline">
                        Register 
                    </a>
                "#
            }
    }
}


pub fn decode(body: &str) -> String{
//Decodes a string from http format (Example space = +, etc...)
//To ascii
    let bytes = body.as_bytes();
    let mut result_bytes: Vec<u8> = Vec::new();
    let mut i = 0;
    while i < bytes.len(){

        if bytes[i] == b'%'{
            if i+2 < bytes.len() {

                let hex = std::str::from_utf8(&bytes[i+1..i+3]).unwrap();

                let byte = u8::from_str_radix(hex, 16).unwrap();

                result_bytes.push(byte);

                
                i = i+3;
            }
            else{
                i+=1;
            }
        }
        else if bytes[i] == b'+' {
            result_bytes.push(b' ');
            i+=1;
        }

        else{
            result_bytes.push(bytes[i]);
            i+=1;
        }
    }

    
    String::from_utf8(result_bytes).unwrap()
}


//Takes in the body, splits the variables on each &, and returns the decoded string

pub fn decode_body(body: &str) -> String{
    let mut comment_body = String::new();

    //Spliting the body:
    for pair in body.split('&'){
        match pair.split_once('='){
            Some(("body", value)) => {
                comment_body = decode(value) ;
            },
            _ => {}
        }
    }


    comment_body

}


pub fn decode_body_field(body: &str, p: &str) -> String{
    let mut field = String::new();

    //Spliting the body:
    for pair in body.split('&'){
        match pair.split_once('='){
            Some((key, value)) => {
                if key == p{
                    field = decode(value) ;
                }
            },
            _ => {}
        }
    }


    field 

}

pub fn encode_html(body: &str) -> String{
    let mut out = String::new();

    for c in body.chars(){
        match c {
            '<' => {
                out.push_str("&lt;");
            },
            '>' => {
                out.push_str("&gt;");
            }
            '&' => {
                out.push_str("&amp;")
            }
            '\'' => {
                out.push_str("&#39;");
            }

            '"' => {
                out.push_str("&quot;");
            }
            _ => {
                out.push(c);
            }
        }
    }


    out
}



pub fn items() -> Vec<PostCard> {
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


pub fn get_username(id: u32) -> &'static str{
    match id {
        0 => "Bogdan",
        1 => "Zeth",
        2 => "Cat",
        3 => "Cats are losers",
        _ => "Weird Id",
    }
}