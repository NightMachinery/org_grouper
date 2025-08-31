use std::process::Command;
use std::fs;
use tempfile::NamedTempFile;
use std::io::Write;

#[test]
fn test_basic_grouping_with_cat() {
    let org_content = "* First\nContent 1\n* Second\nContent 2\n";
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(org_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let output = Command::new("cargo")
        .args(&["run", "--", "--group-headings-at=1", "cat"])
        .stdin(std::fs::File::open(temp_file.path()).unwrap())
        .output()
        .expect("Failed to execute org_grouper");

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // The output should contain both groups separated by newlines (nulls replaced)
    assert!(stdout.contains("* First\nContent 1\n"));
    assert!(stdout.contains("* Second\nContent 2\n"));
}

#[test]
fn test_level_2_grouping() {
    let org_content = "* Top\n** Sub1\nContent1\n** Sub2\nContent2\n";
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(org_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let output = Command::new("cargo")
        .args(&["run", "--", "--group-headings-at=2", "cat"])
        .stdin(std::fs::File::open(temp_file.path()).unwrap())
        .output()
        .expect("Failed to execute org_grouper");

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // With corrected logic: level 2 grouping includes all content, grouping by headings <= 2
    // Should contain all content including the top-level heading
    assert!(stdout.contains("* Top"));
    assert!(stdout.contains("** Sub1\nContent1\n"));
    assert!(stdout.contains("** Sub2\nContent2\n"));
}

#[test]
fn test_no_null_replacement() {
    let org_content = "* First\nContent\n* Second\nMore\n";
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(org_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let output = Command::new("cargo")
        .args(&["run", "--", "--out-replace-nulls=no", "cat"])
        .stdin(std::fs::File::open(temp_file.path()).unwrap())
        .output()
        .expect("Failed to execute org_grouper");

    // With nulls preserved, the output should contain null bytes
    let stdout = output.stdout;
    assert!(stdout.contains(&0u8)); // Should contain null bytes
}

#[test]
fn test_with_grep_command() {
    let org_content = "* TODO First task\nSome details\n* DONE Second task\nOther details\n* TODO Third task\nMore details\n";
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(org_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    // Use ugrep with --null-data to find sections containing "TODO"
    let output = Command::new("cargo")
        .args(&["run", "--", "ugrep", "--null-data", "TODO"])
        .stdin(std::fs::File::open(temp_file.path()).unwrap())
        .output()
        .expect("Failed to execute org_grouper");

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // ugrep --null-data shows full matching records (sections), so we should see complete sections
    assert!(stdout.contains("* TODO First task"));
    assert!(stdout.contains("* TODO Third task")); 
    // Should not contain the DONE section
    assert!(!stdout.contains("* DONE Second task"));
}

#[test]
fn test_error_handling_invalid_level() {
    let org_content = "* Test\nContent\n";
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(org_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let output = Command::new("cargo")
        .args(&["run", "--", "--group-headings-at=invalid", "cat"])
        .stdin(std::fs::File::open(temp_file.path()).unwrap())
        .output()
        .expect("Failed to execute org_grouper");

    assert!(!output.status.success());
}

#[test]
fn test_empty_input() {
    let temp_file = NamedTempFile::new().unwrap();
    // Empty file

    let output = Command::new("cargo")
        .args(&["run", "--", "cat"])
        .stdin(std::fs::File::open(temp_file.path()).unwrap())
        .output()
        .expect("Failed to execute org_grouper");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.trim(), ""); // Should be empty output
}

#[test]
fn test_complex_nested_structure() {
    let org_content = fs::read_to_string("tests/test_data.org")
        .expect("Could not read test_data.org");

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(org_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    // Test level 1 grouping
    let output = Command::new("cargo")
        .args(&["run", "--", "--group-headings-at=1", "wc", "-l"])
        .stdin(std::fs::File::open(temp_file.path()).unwrap())
        .output()
        .expect("Failed to execute org_grouper");

    assert!(output.status.success());
    
    // Test level 2 grouping  
    let output2 = Command::new("cargo")
        .args(&["run", "--", "--group-headings-at=2", "wc", "-l"])
        .stdin(std::fs::File::open(temp_file.path()).unwrap())
        .output()
        .expect("Failed to execute org_grouper");

    assert!(output2.status.success());
}

#[test]
fn test_custom_null_replacement() {
    let org_content = "* First\nContent 1\n* Second\nContent 2\n";
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(org_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    // Test with pipe separator
    let output = Command::new("cargo")
        .args(&["run", "--", "--out-replace-nulls-with=| ", "cat"])
        .stdin(std::fs::File::open(temp_file.path()).unwrap())
        .output()
        .expect("Failed to execute org_grouper");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("* First\nContent 1\n| * Second\nContent 2\n"));
}

#[test]
fn test_tab_replacement() {
    let org_content = "* First\nContent\n* Second\nMore\n";
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(org_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let output = Command::new("cargo")
        .args(&["run", "--", "--out-replace-nulls-with=\\t", "cat"])
        .stdin(std::fs::File::open(temp_file.path()).unwrap())
        .output()
        .expect("Failed to execute org_grouper");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should contain tab character between sections
    assert!(stdout.contains("\t"));
}

#[test]
fn test_empty_replacement() {
    let org_content = "* First\nContent\n* Second\nMore\n";
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(org_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let output = Command::new("cargo")
        .args(&["run", "--", "--out-replace-nulls-with=", "cat"])
        .stdin(std::fs::File::open(temp_file.path()).unwrap())
        .output()
        .expect("Failed to execute org_grouper");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // With empty replacement, sections should be concatenated directly
    assert_eq!(stdout, "* First\nContent\n* Second\nMore\n");
}

#[test]
fn test_level_3_includes_all_higher_levels() {
    let org_content = "Intro\n* Level1\nContent1\n** Level2\nContent2\n*** Level3\nContent3\n**** Level4\nContent4\n** AnotherLevel2\nEnd\n";
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(org_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let output = Command::new("cargo")
        .args(&["run", "--", "--group-headings-at=3", "cat"])
        .stdin(std::fs::File::open(temp_file.path()).unwrap())
        .output()
        .expect("Failed to execute org_grouper");

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should include all content and group by headings of level <= 3
    assert!(stdout.contains("Intro"));  // Initial content
    assert!(stdout.contains("* Level1"));  // Level 1 heading starts new group
    assert!(stdout.contains("** Level2"));  // Level 2 heading starts new group  
    assert!(stdout.contains("*** Level3"));  // Level 3 heading starts new group
    assert!(stdout.contains("**** Level4"));  // Level 4 stays in same group as Level 3
    assert!(stdout.contains("** AnotherLevel2"));  // Another Level 2 starts new group
}

#[test]
fn test_double_dash_separator() {
    let org_content = "* First\nContent 1\n* Second\nContent 2\n";
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(org_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    // Test that -- works to separate our options from command options
    let output = Command::new("cargo")
        .args(&["run", "--", "--group-headings-at=1", "--", "echo", "test-output"])
        .stdin(std::fs::File::open(temp_file.path()).unwrap())
        .output()
        .expect("Failed to execute org_grouper");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test-output"));
}

#[test]  
fn test_command_with_conflicting_option_names() {
    let org_content = "* Test\nContent\n";
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(org_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    // Test passing a command that has its own --help option
    let output = Command::new("cargo")
        .args(&["run", "--", "--", "sh", "-c", "echo 'command executed'"])
        .stdin(std::fs::File::open(temp_file.path()).unwrap())
        .output()
        .expect("Failed to execute org_grouper");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("command executed"));
}

#[test]
fn test_hyphen_values_in_command() {
    let org_content = "* Section\nContent\n";
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(org_content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    // Test that we can pass values starting with hyphens to commands
    let output = Command::new("cargo")
        .args(&["run", "--", "echo", "-n", "no-newline"])
        .stdin(std::fs::File::open(temp_file.path()).unwrap())
        .output()
        .expect("Failed to execute org_grouper");

    let stdout = String::from_utf8_lossy(&output.stdout);
    // echo -n should not add a newline, so content should be concatenated
    assert!(stdout.contains("no-newline"));
    assert!(!stdout.ends_with("\n\n")); // Should not have extra newline from echo -n
}