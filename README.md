# Rust Runner

## Introduction
This project is to execute all the transactions within a group of blocks (e.g 100) again by using the same EVM environment when these blocks are executed. And measure the on-cpu and off-cpu performance during this action.

## Pre-Install
1. Rust and Cargo

2. Maintain a Reth archive node.

3. Perf

4. bcc - offcputime

5. FlameGraph

In this project, FlameGragh is downloaded through git.
```
git clone https://github.com/brendangregg/FlameGraph
```
In this document, FlameGraph is placed at the same level of rust_runner folder.


## Create Project
1. Use cargo tool to update and build the project.
```
cargo update
cargo build
```
2. After cargo build, you can find the exec file named "rust_runner" at ./target/debug folder.

3. If there are any change in main.rs and Cargo.toml, you need to run "cargo build" again in order to crate a new exec file.



## Analyse
1. Run check_rust_runner.sh script, collect the on-cpu and off-cpu data at the same time.
```
sh ckeck_rust_runner.sh
```
2. After run this script, 4 files are created.
