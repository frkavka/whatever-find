use clap::{Arg, Command};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;

use whatever_find::{FileSearcher, SearchMode};

fn main() {
    let matches = Command::new("whatever-find")
        .about(
            "A fast local file search tool with fuzzy matching support - find whatever you need!",
        )
        .version("0.1.0")
        .long_about(
            "A high-performance file search tool with smart pattern detection:
• Auto-detection (default) - automatically detects glob, regex, or substring patterns
• Manual override options available for edge cases

Examples:
  whatever-find config.txt           # Substring search for 'config.txt'
  whatever-find '*.rs'               # Auto-detected glob search for .rs files
  whatever-find '\\.rs$'             # Auto-detected regex search for .rs files
  whatever-find --fuzzy confg        # Force fuzzy search (tolerates typos)
  whatever-find --regex '^test'      # Force regex mode
  whatever-find --glob 'test_*'      # Force glob mode
  whatever-find test -p /home/user   # Search in specific directory
  whatever-find --interactive '*.rs' # Interactive mode to select and open files",
        )
        .arg(
            Arg::new("query")
                .help("Search query")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("path")
                .short('p')
                .long("path")
                .help("Search path (default: current directory)")
                .value_name("PATH"),
        )
        .arg(
            Arg::new("regex")
                .short('r')
                .long("regex")
                .help("Force regex matching (overrides auto-detection)")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("fuzzy")
                .short('f')
                .long("fuzzy")
                .help("Force fuzzy matching (tolerates typos)")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("glob")
                .short('g')
                .long("glob")
                .help("Force glob pattern matching (overrides auto-detection)")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("substring")
                .short('s')
                .long("substring")
                .help("Force substring matching (overrides auto-detection)")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("interactive")
                .short('i')
                .long("interactive")
                .help("Interactive mode - select files to open in explorer")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let query = matches.get_one::<String>("query").unwrap();
    let search_path = matches
        .get_one::<String>("path")
        .map(|s| s.as_str())
        .unwrap_or(".");
    let use_regex = matches.get_flag("regex");
    let use_fuzzy = matches.get_flag("fuzzy");
    let use_glob = matches.get_flag("glob");
    let use_substring = matches.get_flag("substring");
    let interactive = matches.get_flag("interactive");

    let search_modes = [use_regex, use_fuzzy, use_glob, use_substring];
    let active_modes = search_modes.iter().filter(|&&x| x).count();

    if active_modes > 1 {
        eprintln!("Error: Cannot use multiple search modes simultaneously");
        process::exit(1);
    }

    let force_mode = if use_regex {
        Some(SearchMode::Regex)
    } else if use_fuzzy {
        Some(SearchMode::Fuzzy)
    } else if use_glob {
        Some(SearchMode::Glob)
    } else if use_substring {
        Some(SearchMode::Substring)
    } else {
        None // Use auto-detection
    };

    if let Err(e) = run_search(query, search_path, force_mode, interactive) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run_search(
    query: &str,
    path: &str,
    force_mode: Option<SearchMode>,
    interactive: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let searcher = FileSearcher::new();
    let search_path = Path::new(path);

    if let Some(SearchMode::Fuzzy) = force_mode {
        let scored_results = searcher.search_fuzzy(search_path, query)?;
        println!(
            "Searching for '{}' in '{}' using forced fuzzy matching...",
            query, path
        );

        if scored_results.is_empty() {
            println!("No files found matching '{}'", query);
        } else {
            let files: Vec<PathBuf> = scored_results
                .iter()
                .map(|(file, _)| file.clone())
                .collect();
            if interactive {
                println!(
                    "Found {} file(s) (sorted by relevance):",
                    scored_results.len()
                );
                for (i, (file, score)) in scored_results.iter().take(20).enumerate() {
                    println!("  [{}] {} (score: {:.2})", i + 1, file.display(), score);
                }
                handle_interactive_selection(&files)?;
            } else {
                println!(
                    "Found {} file(s) (sorted by relevance):",
                    scored_results.len()
                );
                for (file, score) in scored_results.iter().take(20) {
                    println!("  {} (score: {:.2})", file.display(), score);
                }
            }
        }
        return Ok(());
    }

    let (results, actual_mode) = if let Some(mode) = force_mode {
        let results = searcher.search(search_path, query, mode)?;
        (results, mode)
    } else {
        searcher.search_auto_with_mode(search_path, query)?
    };

    let mode_name = match actual_mode {
        SearchMode::Regex => "regex",
        SearchMode::Glob => "glob",
        SearchMode::Substring => "substring",
        SearchMode::Fuzzy => "fuzzy",
    };

    let detection_text = if force_mode.is_some() {
        format!("forced {}", mode_name)
    } else {
        format!("auto-detected {}", mode_name)
    };

    println!(
        "Searching for '{}' in '{}' using {} matching...",
        query, path, detection_text
    );

    if results.is_empty() {
        println!("No files found matching '{}'", query);
    } else {
        if interactive {
            println!("Found {} file(s):", results.len());
            for (i, file) in results.iter().enumerate() {
                println!("  [{}] {}", i + 1, file.display());
            }
            handle_interactive_selection(&results)?;
        } else {
            println!("Found {} file(s):", results.len());
            for file in results {
                println!("  {}", file.display());
            }
        }
    }

    Ok(())
}

fn handle_interactive_selection(files: &[PathBuf]) -> Result<(), Box<dyn std::error::Error>> {
    if files.is_empty() {
        return Ok(());
    }

    println!();
    println!(
        "Enter number to open in explorer (1-{}), 'a' for all, or 'q' to quit:",
        files.len()
    );
    print!("> ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    match input {
        "q" | "quit" => {
            println!("Goodbye!");
            return Ok(());
        }
        "a" | "all" => {
            for file in files {
                open_in_explorer(file)?;
            }
            return Ok(());
        }
        _ => {
            if let Ok(num) = input.parse::<usize>() {
                if num >= 1 && num <= files.len() {
                    let selected_file = &files[num - 1];
                    open_in_explorer(selected_file)?;
                } else {
                    println!(
                        "Invalid number. Please enter a number between 1 and {}",
                        files.len()
                    );
                }
            } else {
                println!("Invalid input. Please enter a number, 'a' for all, or 'q' to quit.");
            }
        }
    }

    Ok(())
}

fn open_in_explorer(file_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!("Opening {} in explorer...", file_path.display());

    #[cfg(target_os = "windows")]
    {
        let mut cmd = std::process::Command::new("explorer");
        cmd.arg("/select,").arg(file_path);
        cmd.spawn()?;
    }

    #[cfg(target_os = "macos")]
    {
        let mut cmd = std::process::Command::new("open");
        cmd.arg("-R").arg(file_path);
        cmd.spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        // Try different file managers
        let file_managers = ["nautilus", "dolphin", "thunar", "pcmanfm", "xdg-open"];
        let parent = file_path.parent().unwrap_or(Path::new("."));

        for fm in &file_managers {
            let mut cmd = std::process::Command::new(fm);
            if fm == &"xdg-open" {
                cmd.arg(parent);
            } else {
                cmd.arg("--select").arg(file_path);
            }
            if cmd.spawn().is_ok() {
                break;
            }
        }
    }

    Ok(())
}
