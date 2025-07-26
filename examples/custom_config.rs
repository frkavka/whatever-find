//! Example showing how to use custom configuration

use whatever_find::{FileSearcher, Config};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Custom Configuration Example ===\n");

    // Create custom configuration
    let config = Config {
        max_depth: Some(2),           // Only search 2 levels deep
        ignore_hidden: false,         // Include hidden files
        case_sensitive: true,         // Case-sensitive search
        ignore_patterns: vec![
            "target".to_string(),     // Ignore Rust build directory
            "*.tmp".to_string(),      // Ignore temporary files
            ".git".to_string(),       // Ignore git directory
        ],
        max_file_size: Some(1024 * 1024), // Ignore files larger than 1MB
    };

    let searcher = FileSearcher::with_config(config);
    let search_path = Path::new(".");

    println!("Configuration:");
    println!("  - Max depth: 2 levels");
    println!("  - Include hidden files: Yes");
    println!("  - Case sensitive: Yes");
    println!("  - Max file size: 1MB");
    println!("  - Ignore patterns: target, *.tmp, .git");
    println!();

    // Search with custom config
    let results = searcher.search_auto(search_path, "*.rs")?;
    println!("Found {} Rust files with custom configuration:", results.len());
    
    for file in results.iter().take(15) {
        println!("  {}", file.display());
    }

    if results.len() > 15 {
        println!("  ... and {} more files", results.len() - 15);
    }

    println!();

    // Compare with default config
    let default_searcher = FileSearcher::new();
    let default_results = default_searcher.search_auto(search_path, "*.rs")?;
    
    println!("Comparison with default configuration:");
    println!("  Custom config: {} files", results.len());
    println!("  Default config: {} files", default_results.len());
    
    if results.len() != default_results.len() {
        println!("  Difference: {} files", 
                 (default_results.len() as i32 - results.len() as i32).abs());
    }

    Ok(())
}