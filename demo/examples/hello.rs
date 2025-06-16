fn main() {
    // This is a comment
    println!("Hello, world!");

    let x = 42; // A variable
    let mut sum = 0;

    // Loop from 0 to x
    for i in 0..x {
        sum += i;
    }

    println!("Sum of numbers from 0 to {}: {}", x - 1, sum);

    // Using a conditional
    if sum > 100 {
        println!("That's a big sum!");
    } else {
        println!("The sum is reasonable.");
    }

    // Define a struct
    struct Point {
        x: i32,
        y: i32,
    }

    // Create an instance
    let origin = Point { x: 0, y: 0 };

    // Function that takes a Point
    fn distance_from_origin(p: &Point) -> f64 {
        let dx = p.x as f64;
        let dy = p.y as f64;
        (dx * dx + dy * dy).sqrt()
    }

    println!("Distance: {}", distance_from_origin(&origin));
}
