use std::io;

pub fn foo_vec(x: usize, mut s: std::slice::Iter<u8>) -> Option<usize> {
    Some(x + s.position(|&c| c == 's' as u8)? + s.position(|&c| c == 'e' as u8)? + s.position(|&c| c == 'i' as u8)?)
}

pub fn foo(x: usize, ss: &str) -> Option<usize> {
    let ss = ss.as_bytes().to_vec();
     Some(x + ss.iter().position(|&c| c == 's' as u8)? + ss.iter().position(|&c| c == 'e' as u8)? + ss.iter().position(|&c| c == 'i' as u8)?)
    // foo_vec(x, ss.as_bytes().to_vec().iter())
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
