mod combinators;
mod element;

use std::fs::read_to_string;

use combinators::Parser;
use element::element;

fn main() {
    let xml = read_to_string("xml/simple.xml").unwrap();
    let parsed = element().parse(&xml).expect("Couldn't parse this input");
    println!("{:?}", parsed);
}
