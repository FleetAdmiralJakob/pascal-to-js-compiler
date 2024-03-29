use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::{Write};
use regex::Regex;

const INPUT_DIR: &str = "./pascal-input";

// Returns the code inside the two specified words. The words are compared case-insensitive.
fn code_inside_of(full_code: &str, from: &str, to: &str) -> String {
    let code: Vec<&str> = full_code.lines().collect();

    let position_from = code.iter().position(|&code| code.to_lowercase().contains(&from.to_lowercase()));
    let position_to = code.iter().position(|&code| code.to_lowercase().contains(&to.to_lowercase()));

    return match (position_from, position_to) {
        (Some(from_index), Some(to_index)) if from_index < to_index => {
            let sliced_code = &code[from_index + 1..to_index];
            sliced_code.join("\n")
        },
        _ => String::from(full_code),
    }
}

fn check_if_ends_with_semicolon(code: &str, file_name: &str, line_number: usize) {
    if !code.ends_with(';') {
        panic!("Fatal: Syntax error: expected ';' at the end of the function call of program: {file_name} at line: {line_number}");
    }
}

fn str_to_types(s: &str) -> Option<Types> {
    match s {
        "string" => Some(Types::String),
        "integer" => Some(Types::Integer),
        _ => None,
    }
}

struct Content {
    file_content: String,
    file_name: String
}

#[derive(Hash, Eq, PartialEq, Debug)]
enum Types {
    String,
    Integer
}

#[derive(Hash, Eq, PartialEq, Debug)]
struct Variable {
    name: String,
    mutable: bool,
    type_: Types
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
                file_content: fs::read_to_string(&file_name).unwrap(),
                file_name: path.file_name().unwrap().to_str().unwrap().to_string(),
            };

            contents.push(content);
        }
    }
    
    for content in contents {
        let re = Regex::new(r"^\s*program\s+(\w+);?").unwrap();

        let caps = re.captures(&content.file_content);

        if let Some(caps) = caps {
            if !caps.get(0).unwrap().as_str().ends_with(";") {
                panic!("Fatal: Syntax error: expected ';' at the end of program declaration of program: {}", content.file_name);
            }

            let program_name = caps.get(1).unwrap().as_str();

            fs::create_dir_all("js-output").unwrap();

            let mut file = File::create(format!("js-output/{}.js", program_name.to_lowercase())).unwrap();

            let words: Vec<&str> = content.file_content.split_whitespace().collect();

            let mut end_of_library_declaration_position: usize = 2;
            let mut enable_crt: bool = false;

            let mut last_word_of_program_or_library_declaration_code = format!("{program_name};");

            if words[2].to_lowercase() == "uses" {
                let mut i = 3;

                while words[i].ends_with(";") {
                    if words[i].to_lowercase() == "crt;" {
                        enable_crt = true;
                    }
                    last_word_of_program_or_library_declaration_code = words[i].parse().unwrap();
                    i += 1;
                }
                end_of_library_declaration_position = i;
            }

            let mut end_of_variable_declaration_position = end_of_library_declaration_position;

            let variable_declaration_code = code_inside_of(&content.file_content, &last_word_of_program_or_library_declaration_code, "begin");
            let variable_declaration_code_lines = variable_declaration_code.lines();
            let lines_before_variable_declaration_code = content.file_content.lines().take_while(|&line| !line.to_lowercase().contains(&last_word_of_program_or_library_declaration_code.to_lowercase())).count();

            let mut variables: HashSet<Variable> = HashSet::new();

            let mut output_code = String::new();

            for (i, line) in variable_declaration_code_lines.enumerate() {
                let line = line.trim();
                let line_number = lines_before_variable_declaration_code + i + 2;
                if line.starts_with("var") && !line.contains(":") || line.is_empty() {
                    continue;
                }

                let parts: Vec<&str> = line.split(":").collect();
                let variable_name = parts[0].trim().trim_start_matches("var").trim();

                if variables.iter().any(|v| v.name == variable_name) {
                    panic!("Fatal: Syntax error: Variable {variable_name} is declared more than once in program: {} at line: {line_number}", content.file_name);
                }

                let type_and_maybe_value = parts[1].trim();

                let parts: Vec<&str> = type_and_maybe_value.split("=").collect();
                let mut variable_type = parts[0].trim();
                let variable_value = if parts.len() > 1 { parts[1].trim() } else { "" };

                let js_value = if !variable_value.is_empty() {
                    format!(" = {}", variable_value.trim_end_matches(";"))
                } else {
                    if variable_type.ends_with(";") {
                        variable_type = variable_type.trim_end_matches(";");
                    } else {
                        panic!("Fatal: Syntax error: Forgot to put an ; at the end of your variable declaration of program: {}", content.file_name)
                    }
                    String::new()
                };

                let variable_names: Vec<&str> = variable_name.split(',').filter(|s| !s.is_empty()).map(|s| s.trim()).collect();

                for variable_name in &variable_names {
                    if let Some(parsed_variable_type) = str_to_types(variable_type) {
                        variables.insert(Variable { name: variable_name.parse().unwrap(), mutable: true, type_: parsed_variable_type });
                        output_code.push_str(&format!("let {variable_name}{js_value};\n"))
                    } else {
                        panic!("Unsupported variable type: {variable_type} in program: {} at line: {line_number}", content.file_name)
                    }
                }
            }

            while words[end_of_variable_declaration_position].to_lowercase() != "begin" {
                end_of_variable_declaration_position += 1;
            }

            if !(words[end_of_variable_declaration_position].to_lowercase() == "begin") {
                panic!("Fatal: Syntax error: expected 'begin' after program and library declaration of program: {}", content.file_name);
            }

            if !content.file_content.to_lowercase().ends_with("end.") {
                let line_number = content.file_content.lines().count() + 1;
                panic!("Fatal: Syntax error: expected 'end.' at the end of program: {} at line: {line_number}", content.file_name);
            }

            let lines_before_main = content.file_content.lines().take_while(|&line| !line.to_lowercase().contains("begin")).count();
            let main_code = code_inside_of(&content.file_content, "begin", "end.");
            let mut async_behaviour_enabled = false;

            for (i, line) in main_code.lines().enumerate() {
                let trimmed_line = line.trim();
                let line_number = lines_before_main + i + 2;
                if let Some(captures) = Regex::new(r"^(?i)Writeln\((.*?)\);?$").unwrap().captures(trimmed_line) {
                    check_if_ends_with_semicolon(line, &content.file_name, line_number);
                    let message = captures.get(1).unwrap().as_str();
                    if !message.contains("'") && !variables.iter().any(|v| v.name == message) {
                        panic!("Fatal: Syntax error: Variable '{message}' used in Writeln function is not declared in the program: {} at line: {line_number}", content.file_name);
                    }
                    output_code.push_str(&format!("console.log({message});\n"));
                }
                if let Some(captures) = Regex::new(r"^(?i)Delay\((.*?)\);?$").unwrap().captures(trimmed_line) {
                    if !enable_crt {
                        panic!("Fatal: Syntax error: 'Delay' is not defined in this scope. Maybe you missed the declaration of the CRT library.");
                    }
                    check_if_ends_with_semicolon(line, &content.file_name, line_number);
                    let delay = captures.get(1).unwrap().as_str();
                    if !(delay.parse::<i32>().is_ok() || variables.iter().any(|v| v.name == delay && v.type_ == Types::Integer)) {
                        panic!("Fatal: Syntax error: 'Delay' function expects a valid integer or a declared variable as an argument. Found: '{delay}' at line: {line_number}");
                    }
                    async_behaviour_enabled = true;
                    output_code.push_str(&format!("await new Promise(resolve => setTimeout(resolve, {delay}));\n"));
                }
            }

            if async_behaviour_enabled {
                output_code = format!("(async () => {{\n{output_code}}})().catch(console.error);\n");
            }

            file.write_all(output_code.as_bytes()).unwrap();
            file.flush().unwrap()
        } else {
            panic!("Fatal: Syntax error: expected 'program [name];' at the beginning of the program: {}", content.file_name);
        }
    }
}
