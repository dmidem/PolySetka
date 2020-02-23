use std::io;

pub fn foo(x: usize, s: &str) -> Option<usize> {
    // let s = "testing";
    // let ss = s.to_string();
    Some(x + s.find('s')? + s.find('e')? + s.find('i')?)
}

fn main() {
    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();

    let mut x: usize = 0;
    for _i in 0..100_000_000 {
        x = foo(x, &line).unwrap_or(x);
    }
    
    println!("{}", x)
}
