use std::fs::File;
use std::io::Write;
use regex::Regex;

fn remove_code_outside(full_code: String, from: &str, to: &str) -> String {
    let code: Vec<&str> = full_code.lines().collect();

    let position_from = code.iter().position(|&code| code.contains(from));
    let position_to = code.iter().position(|&code| code.contains(to));

    return match (position_from, position_to) {
        (Some(from_index), Some(to_index)) if from_index < to_index => {
            let sliced_code = &code[from_index + 1..to_index];
            sliced_code.join("\n")
        },
        _ => String::from(full_code),
    }
}

fn main() {
    let contents = std::fs::read_to_string("pascal-input/hello.pas").unwrap();

    let first_line = contents.lines().next().unwrap();

    if !(first_line.split_whitespace().nth(0).unwrap() == "program") {
        panic!("First line of file must be a program declaration");
    }

    let program_name = first_line.split_whitespace().nth(1).unwrap();
    if !program_name.ends_with(';') {
        panic!("Fatal: Syntax error: expected ';' at the end of program declaration");
    }

    let program_name = program_name.trim_end_matches(';');

    std::fs::create_dir_all("js-output").unwrap();

    let mut file = File::create(format!("js-output/{}.js", program_name.to_lowercase())).unwrap();

    if !(contents.split_whitespace().nth(2).unwrap().to_lowercase() == "begin") {
        panic!("Fatal: Syntax error: expected 'begin' after program declaration");
    }

    if !contents.ends_with("end.") {
        panic!("Fatal: Syntax error: expected 'end.' at the end of program");
    }

    let main_code = remove_code_outside(contents, "begin", "end.");
    
    for line in main_code.lines() {
        let trimmed_line = line.trim();
        if let Some(captures) = Regex::new(r"^writeln\('(.*?)'\);$").unwrap().captures(trimmed_line) {
            let message = captures.get(1).unwrap().as_str();
            file.write(format!("console.log('{}');\n", message).as_bytes()).unwrap();
        }
    }
    
    file.flush().unwrap();
}
