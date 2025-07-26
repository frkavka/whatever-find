use criterion::{black_box, criterion_group, criterion_main, Criterion};
use file_search::config::Config;
use file_search::indexer::FileIndexer;
use file_search::search::SearchEngine;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn create_test_files(dir: &TempDir, count: usize) -> Result<(), std::io::Error> {
    for i in 0..count {
        let filename = format!("test_file_{}.txt", i);
        let filepath = dir.path().join(&filename);
        fs::write(filepath, format!("content {}", i))?;
    }

    let subdir = dir.path().join("subdir");
    fs::create_dir(&subdir)?;
    for i in 0..count / 2 {
        let filename = format!("nested_file_{}.rs", i);
        let filepath = subdir.join(&filename);
        fs::write(filepath, format!("rust content {}", i))?;
    }

    Ok(())
}

fn benchmark_indexing(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    create_test_files(&temp_dir, 1000).unwrap();

    c.bench_function("index_1000_files", |b| {
        b.iter(|| {
            let config = Config::default();
            let mut indexer = FileIndexer::new(config);
            black_box(
                indexer
                    .build_index(temp_dir.path().to_str().unwrap())
                    .unwrap(),
            )
        })
    });
}

fn benchmark_substring_search(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    create_test_files(&temp_dir, 1000).unwrap();

    let config = Config::default();
    let mut indexer = FileIndexer::new(config.clone());
    let index = indexer
        .build_index(temp_dir.path().to_str().unwrap())
        .unwrap();
    let search_engine = SearchEngine::new(config);

    c.bench_function("substring_search", |b| {
        b.iter(|| black_box(search_engine.search_substring(&index, "test")))
    });
}

fn benchmark_regex_search(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    create_test_files(&temp_dir, 1000).unwrap();

    let config = Config::default();
    let mut indexer = FileIndexer::new(config.clone());
    let index = indexer
        .build_index(temp_dir.path().to_str().unwrap())
        .unwrap();
    let search_engine = SearchEngine::new(config);

    c.bench_function("regex_search", |b| {
        b.iter(|| {
            black_box(
                search_engine
                    .search_regex(&index, r"test_file_\d+\.txt")
                    .unwrap(),
            )
        })
    });
}

criterion_group!(
    benches,
    benchmark_indexing,
    benchmark_substring_search,
    benchmark_regex_search
);
criterion_main!(benches);
