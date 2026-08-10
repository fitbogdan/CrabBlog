




fn main(){
    let mut request_line = "GET / HTTP/1.1";

    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("");


    println!("{} {} {}", request_line, method, path);

    if path == "/" {
        println!("path = /");
    }
    else {
        println!("path = other");
    }




}