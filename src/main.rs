use std::fs;
use std::fs::File;
use std::io::Write;
use regex::Regex;

const INPUT_DIR: &str = "./pascal-input";

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

struct Content {
    file_content: String,
    file_name: String
}

fn main() {
    let files = fs::read_dir(INPUT_DIR).unwrap();
    let files: Vec<_> = files.collect();
    let mut contents: Vec<Content> = Vec::with_capacity(files.len());

    for file in &files {
        let file = file.as_ref().unwrap();
        let path = file.path();
        let file_name = INPUT_DIR.to_owned() + "/" + &*path.file_name().unwrap().to_str().unwrap().to_string();
        if match path.extension() {
            Some(ext) => ext == "pas",
            None => false
        } {
            let content = Content {
                file_content: std::fs::read_to_string(&file_name).unwrap(),
                file_name: path.file_name().unwrap().to_str().unwrap().to_string(),
            };
            
            contents.push(content);
        }
    }
    
    for content in contents {
        let first_line = content.file_content.lines().next().unwrap();

        if !(first_line.split_whitespace().nth(0).unwrap() == "program") {
            panic!("First line of file must be a program declaration of program: {}", content.file_name);
        }

        let program_name = first_line.split_whitespace().nth(1).unwrap();
        if !program_name.ends_with(';') {
            panic!("Fatal: Syntax error: expected ';' at the end of program declaration of program: {}", content.file_name);
        }

        let program_name = program_name.trim_end_matches(';');

        std::fs::create_dir_all("js-output").unwrap();

        let mut file = File::create(format!("js-output/{}.js", program_name.to_lowercase())).unwrap();

        if !(content.file_content.split_whitespace().nth(2).unwrap().to_lowercase() == "begin") {
            panic!("Fatal: Syntax error: expected 'begin' after program declaration of program: {}", content.file_name);
        }

        if !content.file_content.ends_with("end.") {
            panic!("Fatal: Syntax error: expected 'end.' at the end of program: {}", content.file_name);
        }

        let main_code = remove_code_outside(content.file_content, "begin", "end.");

        for line in main_code.lines() {
            let trimmed_line = line.trim();
            if let Some(captures) = Regex::new(r"^writeln\('(.*?)'\);$").unwrap().captures(trimmed_line) {
                let message = captures.get(1).unwrap().as_str();
                file.write(format!("console.log('{}');\n", message).as_bytes()).unwrap();
            }
        }

        file.flush().unwrap();
    }
}
