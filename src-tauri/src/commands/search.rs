use globset::{Glob, GlobSet, GlobSetBuilder};
use memchr::memmem;
use rayon::prelude::*;
use regex::RegexBuilder;
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize)]
pub struct FileMatch {
    pub path: String,
    pub name: String,
    pub score: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TextMatch {
    pub path: String,
    pub line_number: usize,
    pub line_content: String,
    pub column: usize,
    pub match_length: usize,
}

#[derive(Debug, Deserialize)]
pub struct SearchFileOptions {
    pub max_results: Option<usize>,
    pub include_hidden: Option<bool>,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct SearchTextOptions {
    pub max_results: Option<usize>,
    pub case_sensitive: Option<bool>,
    pub is_regex: Option<bool>,
    pub include_hidden: Option<bool>,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub max_file_size: Option<u64>,
}

const DEFAULT_MAX_RESULTS: usize = 500;
const DEFAULT_MAX_FILE_SIZE: u64 = 5 * 1024 * 1024;

static ALWAYS_SKIP: &[&str] = &[
    "node_modules",
    "target",
    ".git",
    "dist",
    "build",
    "out",
    "__pycache__",
    ".next",
    ".cache",
];

fn build_globset(patterns: &[String]) -> Option<GlobSet> {
    if patterns.is_empty() {
        return None;
    }
    let mut builder = GlobSetBuilder::new();
    for p in patterns {
        if let Ok(g) = Glob::new(p) {
            builder.add(g);
        }
    }
    builder.build().ok()
}

fn should_skip(entry: &walkdir::DirEntry, include_hidden: bool) -> bool {
    let name = entry.file_name().to_string_lossy();
    if !include_hidden && name.starts_with('.') {
        return true;
    }
    if entry.file_type().is_dir() && ALWAYS_SKIP.contains(&name.as_ref()) {
        return true;
    }
    false
}

fn fuzzy_score(pattern: &[u8], target: &str) -> Option<i64> {
    if pattern.is_empty() {
        return Some(0);
    }
    let target_bytes = target.as_bytes();
    let mut pi = 0;
    let mut score: i64 = 0;
    let mut consecutive = 0i64;
    let mut prev_match = false;

    for (ti, &tc) in target_bytes.iter().enumerate() {
        if pi < pattern.len() && tc.to_ascii_lowercase() == pattern[pi].to_ascii_lowercase() {
            score += 1;
            if ti == 0 || !target_bytes[ti - 1].is_ascii_alphanumeric() {
                score += 5;
            }
            if tc == pattern[pi] {
                score += 1;
            }
            if prev_match {
                consecutive += 1;
                score += consecutive * 2;
            } else {
                consecutive = 0;
            }
            prev_match = true;
            pi += 1;
        } else {
            prev_match = false;
            consecutive = 0;
        }
    }

    if pi == pattern.len() {
        let len_penalty = (target_bytes.len() as i64 - pattern.len() as i64).min(20);
        Some(score * 100 - len_penalty)
    } else {
        None
    }
}

#[tauri::command]
pub fn search_files(
    root: String,
    pattern: String,
    options: Option<SearchFileOptions>,
) -> Result<Vec<FileMatch>, String> {
    let max_results = options
        .as_ref()
        .and_then(|o| o.max_results)
        .unwrap_or(DEFAULT_MAX_RESULTS);
    let include_hidden = options
        .as_ref()
        .and_then(|o| o.include_hidden)
        .unwrap_or(false);
    let include_set = options
        .as_ref()
        .and_then(|o| o.include.as_deref())
        .and_then(|v| if v.is_empty() { None } else { build_globset(v) });
    let exclude_set = options
        .as_ref()
        .and_then(|o| o.exclude.as_deref())
        .and_then(|v| if v.is_empty() { None } else { build_globset(v) });

    let pattern_bytes = pattern.as_bytes().to_vec();
    let mut scored: Vec<FileMatch> = Vec::with_capacity(max_results * 2);

    for entry in WalkDir::new(&root)
        .follow_links(false)
        .max_depth(20)
        .into_iter()
        .filter_entry(|e| !should_skip(e, include_hidden))
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_dir() {
            continue;
        }

        let path = entry.path();

        if let Some(ref inc) = include_set {
            if !inc.is_match(path) {
                continue;
            }
        }
        if let Some(ref exc) = exclude_set {
            if exc.is_match(path) {
                continue;
            }
        }

        let name = entry.file_name().to_string_lossy();
        let score = match fuzzy_score(&pattern_bytes, &name) {
            Some(s) => s,
            None => continue,
        };

        scored.push(FileMatch {
            path: path.to_string_lossy().into_owned(),
            name: name.into_owned(),
            score,
        });
    }

    scored.sort_unstable_by(|a, b| b.score.cmp(&a.score));
    scored.truncate(max_results);
    Ok(scored)
}

#[tauri::command]
pub fn search_text(
    root: String,
    query: String,
    options: Option<SearchTextOptions>,
) -> Result<Vec<TextMatch>, String> {
    let max_results = options
        .as_ref()
        .and_then(|o| o.max_results)
        .unwrap_or(DEFAULT_MAX_RESULTS);
    let case_sensitive = options
        .as_ref()
        .and_then(|o| o.case_sensitive)
        .unwrap_or(false);
    let is_regex = options.as_ref().and_then(|o| o.is_regex).unwrap_or(false);
    let include_hidden = options
        .as_ref()
        .and_then(|o| o.include_hidden)
        .unwrap_or(false);
    let max_file_size = options
        .as_ref()
        .and_then(|o| o.max_file_size)
        .unwrap_or(DEFAULT_MAX_FILE_SIZE);
    let include_set = options
        .as_ref()
        .and_then(|o| o.include.as_deref())
        .and_then(|v| if v.is_empty() { None } else { build_globset(v) });
    let exclude_set = options
        .as_ref()
        .and_then(|o| o.exclude.as_deref())
        .and_then(|v| if v.is_empty() { None } else { build_globset(v) });

    let use_literal = !is_regex && case_sensitive;
    let literal_finder = if use_literal {
        Some(Arc::new(memmem::Finder::new(query.as_bytes())))
    } else {
        None
    };

    let pattern = if is_regex {
        query.clone()
    } else {
        regex::escape(&query)
    };

    let re = if !use_literal {
        Some(
            RegexBuilder::new(&pattern)
                .case_insensitive(!case_sensitive)
                .build()
                .map_err(|e| format!("Invalid search pattern: {e}"))?,
        )
    } else {
        None
    };

    let files: Vec<_> = WalkDir::new(&root)
        .follow_links(false)
        .max_depth(20)
        .into_iter()
        .filter_entry(|e| !should_skip(e, include_hidden))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            let path = e.path();
            if let Some(ref inc) = include_set {
                if !inc.is_match(path) {
                    return false;
                }
            }
            if let Some(ref exc) = exclude_set {
                if exc.is_match(path) {
                    return false;
                }
            }
            if let Ok(meta) = e.metadata() {
                if meta.len() > max_file_size {
                    return false;
                }
            }
            true
        })
        .collect();

    let hit_count = Arc::new(AtomicUsize::new(0));
    let done = Arc::new(AtomicBool::new(false));

    let all_matches: Vec<Vec<TextMatch>> = files
        .par_iter()
        .filter_map(|entry| {
            if done.load(Ordering::Relaxed) {
                return None;
            }

            let content = fs::read_to_string(entry.path()).ok()?;
            let path_str = entry.path().to_string_lossy().into_owned();
            let mut local = Vec::new();

            if let Some(ref finder) = literal_finder {
                for (line_idx, line) in content.lines().enumerate() {
                    let line_bytes = line.as_bytes();
                    let mut start = 0;
                    while let Some(pos) = finder.find(&line_bytes[start..]) {
                        local.push(TextMatch {
                            path: path_str.clone(),
                            line_number: line_idx + 1,
                            line_content: line.to_string(),
                            column: start + pos,
                            match_length: query.len(),
                        });
                        start += pos + 1;
                        if hit_count.load(Ordering::Relaxed) + local.len() >= max_results {
                            break;
                        }
                    }
                    if hit_count.load(Ordering::Relaxed) + local.len() >= max_results {
                        break;
                    }
                }
            } else if let Some(ref re) = re {
                for (line_idx, line) in content.lines().enumerate() {
                    for m in re.find_iter(line) {
                        local.push(TextMatch {
                            path: path_str.clone(),
                            line_number: line_idx + 1,
                            line_content: line.to_string(),
                            column: m.start(),
                            match_length: m.end() - m.start(),
                        });
                        if hit_count.load(Ordering::Relaxed) + local.len() >= max_results {
                            break;
                        }
                    }
                    if hit_count.load(Ordering::Relaxed) + local.len() >= max_results {
                        break;
                    }
                }
            }

            if !local.is_empty() {
                let prev = hit_count.fetch_add(local.len(), Ordering::Relaxed);
                if prev + local.len() >= max_results {
                    done.store(true, Ordering::Relaxed);
                    local.truncate(max_results.saturating_sub(prev));
                }
                Some(local)
            } else {
                None
            }
        })
        .collect();

    let total: usize = all_matches.iter().map(|v| v.len()).sum();
    let mut results = Vec::with_capacity(total.min(max_results));
    for batch in all_matches {
        let remaining = max_results.saturating_sub(results.len());
        if remaining == 0 {
            break;
        }
        results.extend(batch.into_iter().take(remaining));
    }

    Ok(results)
}
