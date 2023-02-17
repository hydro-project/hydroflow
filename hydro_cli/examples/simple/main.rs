fn main() {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let trimmed = input.trim();
    println!("Hello, world, I got {trimmed}!");
}
