use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        println!();
        return;
    }

    let (no_newline, text_args) = if args[0] == "-n" {
        (true, &args[1..])
    } else {
        (false, &args[..])
    };

    let output = text_args.join(" ");

    if no_newline {
        print!("{}", output);
    } else {
        println!("{}", output);
    }
}
