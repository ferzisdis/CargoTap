// Test file for CargoTap file switching feature
// This is a simple Rust file to demonstrate loading different files

fn main() {
    println!("Hello from test file!");
    let x = 42;
    let y = x + 8;
    println!("The answer is: {}", y);
}

fn greet(name: &str) {
    println!("Hello, {}!", name);
}

struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }

    fn distance_from_origin(&self) -> f64 {
        ((self.x.pow(2) + self.y.pow(2)) as f64).sqrt()
    }
}
