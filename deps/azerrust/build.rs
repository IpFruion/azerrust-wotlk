use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use glob::glob;
use rayon::iter::{ParallelBridge, ParallelIterator};

fn main() {
    // So that cargo build can work
    if cfg!(debug_assertions) {
        return;
    }
    let mut includes = Vec::new();

    // Need our own include/ for azerrust_helpers.h
    includes.push(PathBuf::from("include"));

    add_includes("../../src/", &mut includes);
    add_includes("../../deps/g3dlite/", &mut includes);
    add_includes("../../deps/recastnavigation/", &mut includes);
    add_includes("../../deps/fmt/", &mut includes);

    // boost is a system dependency found by CMake: pass its include path via
    // the CXXFLAGS env var (respected by the cc crate automatically), or
    // via the AZERRUST_EXTRA_INCLUDE_DIRS env var.
    if let Ok(extra) = std::env::var("AZERRUST_EXTRA_INCLUDE_DIRS") {
        for dir in extra.split(':') {
            if !dir.is_empty() {
                includes.push(PathBuf::from(dir));
            }
        }
    }

    // All bridges: root-level files, conditions, and entities
    let bridges = glob("src/**/*.rs")
        .unwrap()
        .filter_map(|res| res.ok())
        .par_bridge()
        .filter_map(|p| {
            File::open(&p).ok().map(BufReader::new).and_then(|r| {
                r.lines()
                    .take(40)
                    .any(|l| l.is_ok_and(|l| l.contains("#[cxx::bridge]")))
                    .then_some(p)
            })
        })
        .collect::<Vec<_>>();
    println!("cargo:warning={bridges:?}");
    cxx_build::bridges(bridges)
        .includes(&includes)
        .std("gnu++20")
        .compile("azerrust");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=include/azerrust_helpers.h");
}

fn add_includes<P: AsRef<Path>>(root: P, includes: &mut Vec<PathBuf>) {
    let Ok(dir) = std::fs::read_dir(root.as_ref()) else {
        return;
    };
    let mut stack = vec![dir];
    while let Some(current) = stack.pop() {
        for child in current.flatten() {
            let path = child.path();
            if path.is_dir() {
                if let Ok(read_dir) = std::fs::read_dir(&path) {
                    stack.push(read_dir);
                }
                includes.push(path);
            }
        }
    }
}
