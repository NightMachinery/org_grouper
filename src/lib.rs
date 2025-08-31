use regex::Regex;

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

pub fn process_escape_sequences(input: &str) -> String {
    input
        .replace("\\n", "\n")
        .replace("\\t", "\t")
        .replace("\\r", "\r")
        .replace("\\0", "\0")
        .replace("\\\\", "\\")
}

pub fn replace_nulls_in_bytes(input: &[u8], replacement: &str) -> Vec<u8> {
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