// Demo code to showcase different text rendering capabilities
// This demonstrates various programming constructs with syntax highlighting

// Header comment - demonstrates comments
/*
 * Multi-line comment example
 * Shows different comment styles
 */

// Function definitions
fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

// String literals and println! macro
fn greet_user(name: &str) {
    println!("Hello, {}!", name);
    println!("Welcome to CargoTap!");
}

// Control flow with if-else
fn check_number(x: i32) -> &'static str {
    if x > 0 {
        "positive"
    } else if x < 0 {
        "negative"
    } else {
        "zero"
    }
}

// Loop constructs
fn count_to_ten() {
    for i in 1..=10 {
        println!("Count: {}", i);
    }

    let mut counter = 0;
    while counter < 5 {
        println!("While loop: {}", counter);
        counter += 1;
    }
}

// Error handling with Result
fn divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Err("Division by zero".to_string())
    } else {
        Ok(a / b)
    }
}

// Pattern matching
fn process_option(opt: Option<i32>) {
    match opt {
        Some(value) => println!("Got value: {}", value),
        None => println!("No value"),
    }
}

// Struct definition
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }

    fn distance_from_origin(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

// Vector operations
fn vector_demo() {
    let mut numbers = vec![1, 2, 3, 4, 5];
    numbers.push(6);

    let doubled: Vec<i32> = numbers.iter().map(|x| x * 2).collect();

    println!("Doubled: {:?}", doubled);
}

// Main function
fn main() {
    greet_user("Rustacean");

    let result = fibonacci(8);
    println!("Fibonacci(8) = {}", result);

    let status = check_number(-5);
    println!("Number status: {}", status);

    count_to_ten();

    match divide(10.0, 3.0) {
        Ok(result) => println!("10 / 3 = {:.2}", result),
        Err(e) => println!("Error: {}", e),
    }

    let point = Point::new(3.0, 4.0);
    println!("Distance: {:.2}", point.distance_from_origin());

    vector_demo();

    process_option(Some(42));
    process_option(None);
}
