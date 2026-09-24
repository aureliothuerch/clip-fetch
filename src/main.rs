use std::{env, println};

fn main() {
    let args: Vec<String> = env::args().collect();

    println!("{}", args[0]);    
}
