use std::{env, println};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        if !args[1].contains("https://") {
            println!("This is an invalid link")
        }
        else {
            println!("The URL is: {}", args[1]);
        }
    }
    else {
        println!("Please put a link: cf <link>")
    }
}
