use std::f32::consts::GOLDEN_RATIO;

use crate::datatypes::{PostCard};

pub const COMPONENTS_PATH: &str = "static/components/";
pub const AUTH_COMMENT_BOX: &str = "static/components/auth_comment_box.html";
pub const GUEST_COMMENT_BOX: &str = "static/components/guest_comment_box.html";
pub const COMMENT_HTML: &str = "static/components/comment.html";
pub const POST_CARD: &str = "static/components/post_card.html";
pub const SUBCOMMENT_HTML: &str = "static/components/subcomment.html";
pub const DELETE_POST_BUTTON: &str = "static/components/delete_post_button.html";
pub const DELTE_COMMENT_BUTTON: &str = "static/components/delete_comment.html";
pub const DELETE_SUBCOMMENT_BUTTON: &str = "static/components/delete_subcomment.html";
pub const USER_CARD: &str = "static/components/user_card.html";


pub fn get_color_from_id(user_id: u32) -> &'static str{

    const AVATAR_COLORS: [&str; 8] = [    
        "#c94f4f",
        "#c98b4f",
        "#c9c44f",
        "#6fc94f",
        "#4fc9b0",
        "#4f8bc9",
        "#7a4fc9",
        "#c94fa8",
    ];

    const GOLDEN_RATIO: f64 = 1.61803398875;
    const TWO_POW_32: f64 = 2u64.pow(32) as f64;
    const FACTOR: u32 = (TWO_POW_32 / GOLDEN_RATIO) as u32;

    let position = user_id.wrapping_mul(FACTOR);
    let slice = (position>>29) as usize; //Need only 0 to 7 (Only 8 colors), so we only need 3 bits. The 3 highest bits are the most randomised, they are farthest apart from eachother

    /*

        Mindset: Think of the user taking user_id steps of a length X on a circle with circumference = 1.

        We split that circle into AVATAR_COLORS.len() regions, all representing a color. So if you are 0.1 away from the start, you may land in region 1, meaning the first color.

        Now, in order to make this evenly distributed, we need a number, which, never meets itself on the circle, when we add it to itself.

        So it means that no matter how many times you add it to itself, the fractional part will never be the same.

        But more than that. The fractional part never comes CLOSE to another past fractional part, no matter how much you add.

        If the fractional part would come close to another part, the points will all clump up in 1-2 regions and everyone has the same colors.

        The golden ratio is the number that stays furthest from repeating, so every new step lands in one of the largest remaining gaps.



        Long story short its the Golden Ration.



        This is the slower version. The one up is for faster computation. This is for clarity:

        let position = (user_id as f64 * 1.61803398875).fract(); // Position from one to zero on the circle.

        let slice = (position * 8.0) as usize;

        Basically we take the Golden Ratio, and multiply it by our user id, and that way we see where we would land after
        user_id steps of length golden ratio.

        Strip the integer part and keep the decimals with .fract(), to see how much percentage of the circle we since the start we have covered.

        Since circle is len 1, the integer part is just N laps around the circle

        Then just multiply that by 8, because the circle is length one, the fractional distance on it of say, 0.x, is the same as saying X% of the circle.

        So multiply by 8 and boom you get the region of the circle you want, aka the color.
    
    */ 

    AVATAR_COLORS[slice]
}



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