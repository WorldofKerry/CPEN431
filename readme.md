# CPEN 431 in Rust

Follows the assignments the 2024W2 CPEN 431 offering at UBC, but uses Rust instead of Java.

https://docs.google.com/document/d/1AyHvEb7SATx0uo9NOAgmOImCMGCAyjaQ_KL2tOXWNUQ/edit?tab=t.0

http://52.27.39.26:43104/leaderboard

## Setup

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

sudo apt install openjdk-17-jdk openjdk-17-jre
```

## Run

```bash
# Server
cargo run --release --bin cpen431 127.0.0.1 16401
PERF=/usr/lib/linux-tools/5.15.0-130-generic/perf flamegraph -- target/release/cpen431 0.0.0.0 16401

# Clients
java -jar ./a4_2025_dummy_tests_v1.jar --servers-list servers_list.txt
java -jar ./a4_2025_basic_tests_v1.jar --servers-list servers_list.txt
java -jar ./a4_2025_eval_test_v1.jar --servers-list servers_list.txt
java -jar ./a4_2025_eval_test_v1.jar --servers-list servers_list.txt --submit --student-id 12345678
```