use clap::{Arg, Command};
use std::process;
use std::path::Path;

use file_search::{FileSearcher, SearchMode};

fn main() {
    let matches = Command::new("file-search")
        .about("A fast local file search tool with fuzzy matching support")
        .version("0.1.0")
        .long_about("A high-performance file search tool with smart pattern detection:
• Auto-detection (default) - automatically detects glob, regex, or substring patterns
• Manual override options available for edge cases

Examples:
  file-search config.txt           # Substring search for 'config.txt'
  file-search '*.rs'               # Auto-detected glob search for .rs files
  file-search '\\.rs$'             # Auto-detected regex search for .rs files
  file-search --fuzzy confg        # Force fuzzy search (tolerates typos)
  file-search --regex '^test'      # Force regex mode
  file-search --glob 'test_*'      # Force glob mode
  file-search test -p /home/user   # Search in specific directory")
        .arg(
            Arg::new("query")
                .help("Search query")
                .required(true)
                .index(1)
        )
        .arg(
            Arg::new("path")
                .short('p')
                .long("path")
                .help("Search path (default: current directory)")
                .value_name("PATH")
        )
        .arg(
            Arg::new("regex")
                .short('r')
                .long("regex")
                .help("Force regex matching (overrides auto-detection)")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("fuzzy")
                .short('f')
                .long("fuzzy")
                .help("Force fuzzy matching (tolerates typos)")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("glob")
                .short('g')
                .long("glob")
                .help("Force glob pattern matching (overrides auto-detection)")
                .action(clap::ArgAction::SetTrue)
        )
        .arg(
            Arg::new("substring")
                .short('s')
                .long("substring")
                .help("Force substring matching (overrides auto-detection)")
                .action(clap::ArgAction::SetTrue)
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

    if let Err(e) = run_search(query, search_path, force_mode) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run_search(query: &str, path: &str, force_mode: Option<SearchMode>) -> Result<(), Box<dyn std::error::Error>> {
    let searcher = FileSearcher::new();
    let search_path = Path::new(path);
    
    if let Some(SearchMode::Fuzzy) = force_mode {
        let scored_results = searcher.search_fuzzy(search_path, query)?;
        println!("Searching for '{}' in '{}' using forced fuzzy matching...", query, path);
        
        if scored_results.is_empty() {
            println!("No files found matching '{}'", query);
        } else {
            println!("Found {} file(s) (sorted by relevance):", scored_results.len());
            for (file, score) in scored_results.iter().take(20) {
                println!("  {} (score: {:.2})", file.display(), score);
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

    println!("Searching for '{}' in '{}' using {} matching...", query, path, detection_text);

    if results.is_empty() {
        println!("No files found matching '{}'", query);
    } else {
        println!("Found {} file(s):", results.len());
        for file in results {
            println!("  {}", file.display());
        }
    }

    Ok(())
}