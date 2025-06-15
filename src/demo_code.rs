/*
 * Демонстрационный код для отображения
 */
fn main() {
    println!("Hello CodeFlow!");
    let x = 42;
    match x {
        0..=10 => println!("Small"),
        11..=100 => println!("Medium"),
        _ => println!("Large"),
    }
}
