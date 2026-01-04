use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        println!();
        return;
    }

    let no_newline = args[0] == "-n";
    let text_args = if no_newline {
        &args[1..]
    } else {
        &args[..]
    };

    let output = text_args.join(" ");

    if no_newline {
        print!("{}", output);
    } else {
        println!("{}", output);
    }
}
