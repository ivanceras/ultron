// This is a comment, and is ignored by the compiler

fn hello() {
    // Statements here are executed when the compiled binary is called

    // Print text to the console
    println!("Hello World!");

    let raw = "This is a string
        that spans
        multiple lines";
}

fn main() {
    let n = 5;

    if n < 0 {
        print!("{} is negative", n);
    } else if n > 0 {
        print!("{} is positive", n);
    } else {
        print!("{} is zero", n);
    }

    let big_n = if n < 10 && n > -10 {
        println!(", and is a small number, increase ten-fold");

        // This expression returns an `i32`.
        10 * n
    } else {
        println!(", and is a big number, halve the number");

        // This expression must return an `i32` as well.
        n / 2
        // TODO ^ Try suppressing this expression with a semicolon.
    };
    //   ^ Don't forget to put a semicolon here! All `let` bindings need it.

    println!("{} -> {}", n, big_n);
}

/// unicode and cjk
///      ╔═╦╗      .------------.
///      ╠═╬╣      |  文件系统  |
///      ╚═╩╝      '------------'
///
