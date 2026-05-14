use cron::Schedule;
use std::str::FromStr;

fn main() {
    let expr = "0 0 0 * * * *";
    match Schedule::from_str(expr) {
        Ok(_) => println!("Valid!"),
        Err(e) => println!("Error: {}", e),
    }

    let expr = "0 0 * * * *";
    match Schedule::from_str(expr) {
        Ok(_) => println!("Valid 6!"),
        Err(e) => println!("Error 6: {}", e),
    }

    let expr = "* * * * * *";
    match Schedule::from_str(expr) {
        Ok(_) => println!("Valid 6 stars!"),
        Err(e) => println!("Error 6 stars: {}", e),
    }
}
