use org_grouper::{group_org_sections, process_escape_sequences, replace_nulls_in_bytes};

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