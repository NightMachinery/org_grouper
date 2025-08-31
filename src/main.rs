use clap::{Arg, Command as ClapCommand};
use regex::Regex;
use std::ffi::OsStr;
use std::io::{self, Read, Write};
use std::process::{Command, Stdio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = ClapCommand::new("org_grouper")
        .about("Group org-mode sections at a given heading level, separate groups by NUL, pipe to CMD, then optionally replace NULs in CMD output with newlines.")
        .arg(
            Arg::new("group_headings_at")
                .long("group-headings-at")
                .num_args(1)
                .value_name("LEVEL")
                .help("Org heading star level to group at (e.g., 1 for '*', 2 for '**')")
                .default_value("1"),
        )
        .arg(
            Arg::new("out_replace_nulls")
                .long("out-replace-nulls")
                .num_args(1)
                .value_name("yes|no")
                .help("If 'yes', replace NUL (\\0) in CMD output with replacement string")
                .default_value("yes"),
        )
        .arg(
            Arg::new("out_replace_nulls_with")
                .long("out-replace-nulls-with")
                .num_args(1)
                .value_name("STRING")
                .help("String to replace NUL (\\0) characters with (supports escape sequences like \\n, \\t)")
                .default_value("\\n"),
        )
        .arg(
            Arg::new("cmd")
                .num_args(1..)
                .trailing_var_arg(true)
                .required(true)
                .value_name("CMD ..."),
        )
        .get_matches();

    let level: usize = matches
        .get_one::<String>("group_headings_at")
        .unwrap()
        .parse()
        .map_err(|_| "Invalid --group-headings-at value")?;

    let out_replace_nulls = matches
        .get_one::<String>("out_replace_nulls")
        .map(|s| matches!(s.as_str(), "yes" | "true" | "1"))
        .unwrap_or(true);

    let replacement_string = matches
        .get_one::<String>("out_replace_nulls_with")
        .unwrap()
        .clone();
    
    // Process escape sequences in the replacement string
    let processed_replacement = process_escape_sequences(&replacement_string);

    let cmd_parts: Vec<String> = matches
        .get_many::<String>("cmd")
        .unwrap()
        .map(|s| s.to_string())
        .collect();

    let (cmd_prog, cmd_args) = cmd_parts
        .split_first()
        .ok_or("Missing CMD to execute")?;

    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let groups = group_org_sections(&input, level)?;

    let grouped = groups.join("\0");
    let mut child = Command::new(OsStr::new(cmd_prog))
        .args(cmd_args.iter().map(|s| OsStr::new(s)))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    {
        let stdin = child.stdin.as_mut().ok_or("Failed to open CMD stdin")?;
        stdin.write_all(grouped.as_bytes())?;
    }

    let output = child.wait_with_output()?;

    let mut stdout = output.stdout;
    let mut stderr = output.stderr;
    if out_replace_nulls {
        stdout = replace_nulls_in_bytes(&stdout, &processed_replacement);
        stderr = replace_nulls_in_bytes(&stderr, &processed_replacement);
    }

    io::stdout().write_all(&stdout)?;
    io::stderr().write_all(&stderr)?;

    std::process::exit(output.status.code().unwrap_or(1));
}

pub fn group_org_sections(input: &str, level: usize) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let headline_re = Regex::new(r"(?m)^(?P<stars>\*+)\s")?;

    let mut groups: Vec<String> = Vec::new();
    let mut current = String::new();

    for line in input.split_inclusive('\n') {
        if let Some(caps) = headline_re.captures(line) {
            let star_count = caps.name("stars").unwrap().as_str().len();

            // Start a new group when we see a heading at or above the target level
            if star_count <= level {
                // Close current group if it has content
                if !current.is_empty() {
                    groups.push(current);
                    current = String::new();
                }
            }
        }
        
        // Always include content in the current group
        current.push_str(line);
    }
    
    // Add the final group if it has content
    if !current.is_empty() {
        groups.push(current);
    }

    Ok(groups)
}

fn process_escape_sequences(input: &str) -> String {
    input
        .replace("\\n", "\n")
        .replace("\\t", "\t")
        .replace("\\r", "\r")
        .replace("\\0", "\0")
        .replace("\\\\", "\\")
}

fn replace_nulls_in_bytes(input: &[u8], replacement: &str) -> Vec<u8> {
    let replacement_bytes = replacement.as_bytes();
    let mut result = Vec::new();
    
    for &byte in input {
        if byte == 0 {
            result.extend_from_slice(replacement_bytes);
        } else {
            result.push(byte);
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_level_1_heading() {
        let input = "* First heading\nSome content\nMore content\n";
        let groups = group_org_sections(input, 1).unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0], "* First heading\nSome content\nMore content\n");
    }

    #[test]
    fn test_multiple_level_1_headings() {
        let input = "* First heading\nContent 1\n* Second heading\nContent 2\n";
        let groups = group_org_sections(input, 1).unwrap();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0], "* First heading\nContent 1\n");
        assert_eq!(groups[1], "* Second heading\nContent 2\n");
    }

    #[test]
    fn test_nested_headings() {
        let input = "* Top level\n** Sub heading\nSub content\n*** Deep heading\nDeep content\n* Another top\nMore content\n";
        let groups = group_org_sections(input, 1).unwrap();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0], "* Top level\n** Sub heading\nSub content\n*** Deep heading\nDeep content\n");
        assert_eq!(groups[1], "* Another top\nMore content\n");
    }

    #[test]
    fn test_level_2_grouping() {
        let input = "* Top level\n** First sub\nSub content 1\n** Second sub\nSub content 2\n* Another top\n** Third sub\nSub content 3\n";
        let groups = group_org_sections(input, 2).unwrap();
        assert_eq!(groups.len(), 5);
        // First group: everything up to first * (should be empty since we start with *)
        // Second group: * Top level up to ** First sub
        assert_eq!(groups[0], "* Top level\n");
        assert_eq!(groups[1], "** First sub\nSub content 1\n");
        assert_eq!(groups[2], "** Second sub\nSub content 2\n");
        assert_eq!(groups[3], "* Another top\n");
        assert_eq!(groups[4], "** Third sub\nSub content 3\n");
    }

    #[test]
    fn test_empty_input() {
        let input = "";
        let groups = group_org_sections(input, 1).unwrap();
        assert_eq!(groups.len(), 0);
    }

    #[test]
    fn test_no_matching_headings() {
        let input = "Just some text\nwith no headings\n";
        let groups = group_org_sections(input, 1).unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0], input);
    }

    #[test]
    fn test_level_2_includes_all_content() {
        let input = "Some text\n* Level 1\nContent\n** Level 2\nMore content\n";
        let groups = group_org_sections(input, 2).unwrap();
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0], "Some text\n");  // Content before first heading
        assert_eq!(groups[1], "* Level 1\nContent\n");  // Level 1 heading triggers new group
        assert_eq!(groups[2], "** Level 2\nMore content\n");  // Level 2 heading triggers new group
    }

    #[test]
    fn test_level_3_grouping() {
        let input = "Intro\n* L1\nContent\n** L2\nMore\n*** L3\nDeep\n**** L4\nDeeper\n** Another L2\nEnd\n";
        let groups = group_org_sections(input, 3).unwrap();
        assert_eq!(groups.len(), 5);
        assert_eq!(groups[0], "Intro\n");  // Content before first heading
        assert_eq!(groups[1], "* L1\nContent\n");  // L1 triggers new group
        assert_eq!(groups[2], "** L2\nMore\n");  // L2 triggers new group
        assert_eq!(groups[3], "*** L3\nDeep\n**** L4\nDeeper\n");  // L3 triggers new group, L4 stays in same group
        assert_eq!(groups[4], "** Another L2\nEnd\n");  // L2 triggers new group
    }

    #[test]
    fn test_process_escape_sequences() {
        assert_eq!(process_escape_sequences("\\n"), "\n");
        assert_eq!(process_escape_sequences("\\t"), "\t");
        assert_eq!(process_escape_sequences("\\r"), "\r");
        assert_eq!(process_escape_sequences("\\0"), "\0");
        assert_eq!(process_escape_sequences("\\\\"), "\\");
        assert_eq!(process_escape_sequences("hello\\nworld"), "hello\nworld");
        assert_eq!(process_escape_sequences("tab\\there"), "tab\there");
    }

    #[test]
    fn test_replace_nulls_in_bytes() {
        let input = b"hello\0world\0test";
        let result = replace_nulls_in_bytes(input, "\n");
        assert_eq!(result, b"hello\nworld\ntest");
        
        let result = replace_nulls_in_bytes(input, " | ");
        assert_eq!(result, b"hello | world | test");
        
        let result = replace_nulls_in_bytes(input, "");
        assert_eq!(result, b"helloworldtest");
    }
}