# Examples

This directory contains example org files to test the org_grouper functionality.

## Usage Examples

### Basic grouping at level 1:
```bash
cat examples/meeting_notes.org | cargo run -- --group-headings-at=1 ugrep --null-data -i "bug"
```

### Group by level 2 sections:
```bash  
cat examples/meeting_notes.org | cargo run -- --group-headings-at=2 wc -l
```

### Search for TODO items across sections:
```bash
cat examples/meeting_notes.org | cargo run -- --group-headings-at=1 ugrep --null-data "\[.\]"
```

### Keep null separators (useful for further processing):
```bash
cat examples/meeting_notes.org | cargo run -- --out-replace-nulls=no ugrep --null-data "Priority" | wc -c
```

### Using -- separator for commands with conflicting options:
```bash
# When your command has options that might conflict with org_grouper's options
cat examples/meeting_notes.org | cargo run -- --group-headings-at=2 -- grep -E "Priority|Important"

# Pass complex arguments safely to subcommands
cat examples/meeting_notes.org | cargo run -- -- ugrep --null-data --color=always "Bob.*Alice"

# Ensure proper parsing when command starts with dashes
cat examples/meeting_notes.org | cargo run -- -- sh -c "grep TODO && echo 'Found todos'"
```

### Custom null replacement examples:

#### Use pipe separator between sections:
```bash
cat examples/meeting_notes.org | cargo run -- --out-replace-nulls-with="| " ugrep --null-data "Bob"
```

#### Use tab separator:
```bash  
cat examples/meeting_notes.org | cargo run -- --out-replace-nulls-with="\t" wc -l
```

#### Use double newlines for clear section separation:
```bash
cat examples/meeting_notes.org | cargo run -- --out-replace-nulls-with="\n\n" cat
```

#### Remove separators completely (concatenate sections):
```bash
cat examples/meeting_notes.org | cargo run -- --out-replace-nulls-with="" ugrep --null-data -c "Priority"
```