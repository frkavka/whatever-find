//! Basic usage example for the file-search library

use file_search::{FileSearcher, SearchMode};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let searcher = FileSearcher::new();
    let search_path = Path::new(".");

    println!("=== File Search Library Examples ===\n");

    // Example 1: Auto-detection
    println!("1. Auto-detection examples:");
    
    // This will be detected as glob
    let (results, mode) = searcher.search_auto_with_mode(search_path, "*.rs")?;
    println!("   Query: '*.rs' -> Detected as: {:?}", mode);
    println!("   Found {} Rust files", results.len());

    // This will be detected as regex
    let (results, mode) = searcher.search_auto_with_mode(search_path, r"\.toml$")?;
    println!("   Query: r'\\.toml$' -> Detected as: {:?}", mode);
    println!("   Found {} TOML files", results.len());

    // This will be detected as substring
    let (results, mode) = searcher.search_auto_with_mode(search_path, "main")?;
    println!("   Query: 'main' -> Detected as: {:?}", mode);
    println!("   Found {} files containing 'main'", results.len());

    println!();

    // Example 2: Manual mode selection
    println!("2. Manual mode selection:");
    
    let results = searcher.search(search_path, "src", SearchMode::Substring)?;
    println!("   Substring search for 'src': {} files", results.len());

    let results = searcher.search(search_path, "*.md", SearchMode::Glob)?;
    println!("   Glob search for '*.md': {} files", results.len());

    println!();

    // Example 3: Fuzzy search
    println!("3. Fuzzy search example:");
    let fuzzy_results = searcher.search_fuzzy(search_path, "cargp")?; // Typo in "cargo"
    println!("   Fuzzy search for 'cargp' (typo in 'cargo'):");
    for (file, score) in fuzzy_results.iter().take(5) {
        println!("     {} (score: {:.2})", file.file_name().unwrap().to_string_lossy(), score);
    }

    println!();

    // Example 4: Show some actual results
    println!("4. Sample search results:");
    let results = searcher.search_auto(search_path, "*.rs")?;
    println!("   Rust files in current directory:");
    for file in results.iter().take(10) {
        if let Some(name) = file.file_name() {
            println!("     {}", name.to_string_lossy());
        }
    }

    if results.len() > 10 {
        println!("     ... and {} more", results.len() - 10);
    }

    Ok(())
}