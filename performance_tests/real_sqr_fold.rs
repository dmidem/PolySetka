use std::io;

type Real = f64;

pub fn foo(x: Real, s: &Vec::<Real>) -> Real {
    s.iter().fold(x, |sum, si| sum + si * si)
}

fn main() {
    let s: Vec::<Real> = vec![ 1.5, 2.5, 3.5, 4.5, 5.5, 1.5, 2.5, 3.5, 4.5, 5.5, 1.5, 2.5, 3.5, 4.5, 5.5, 1.5, 2.5, 3.5, 4.5, 5.5 ];

    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();

    let mut x: Real = line.trim().parse().unwrap();

    for _i in 0..100_000_000 {
        x = foo(x, &s);
    }
    
    println!("{}", x)
}
