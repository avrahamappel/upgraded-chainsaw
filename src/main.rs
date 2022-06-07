// mod combinators;
// mod element;
#![feature(type_name_of_val)]
// use std::fs::read_to_string;

use std::any::type_name_of_val;

// use combinators::Parser;
// use element::element;
//
fn foo() -> impl Fn() -> Result<u32, u32> {
    || match Ok(1) {
        ok @ Ok(_) => {
            return ok;
        }
        e @ Err(_) => {
            return e;
        }
    }
}

fn main() {
    // let xml = read_to_string("xml/simple.xml").unwrap();
    // let parsed = element().parse(&xml).expect("Couldn't parse this input");
    // println!("{:?}", parsed);
    println!("{:?}", type_name_of_val(&foo()));
}
