use clap::{Arg, Command as ClapCommand};
use std::ffi::OsStr;
use std::io::{self, Read, Write};
use std::process::{Command, Stdio};
use org_grouper::{group_org_sections, process_escape_sequences, replace_nulls_in_bytes};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = ClapCommand::new("org_grouper")
        .about("Group org-mode sections at a given heading level, separate groups by NUL, pipe to CMD, then optionally replace NULs in CMD output with newlines.")
        .after_help("EXAMPLES:\n  \
            cat notes.org | org_grouper ugrep --null-data \"TODO\"\n  \
            cat notes.org | org_grouper --group-headings-at=2 -- grep -E \"Priority|Important\"\n  \
            cat notes.org | org_grouper --out-replace-nulls-with=\"\\n---\\n\" -- wc -l")
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
                .allow_hyphen_values(true)
                .required(true)
                .value_name("CMD ...")
                .help("Command to execute with grouped org sections as input"),
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