use std::{net::Ipv4Addr, process::Command};

struct ChildGuard {
    pub child: std::process::Child,
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        self.child.kill().unwrap();
    }
}

fn parse_servers_list() -> Vec<(Ipv4Addr, u16)> {
    let servers_list = include_str!("../servers_list.txt");
    servers_list
        .lines()
        .map(|s| {
            let mut parts = s.split(':');
            let ip = parts.next().unwrap().parse().unwrap();
            let port = parts.next().unwrap().parse().unwrap();
            (ip, port)
        })
        .collect()
}

fn check_log_file(path: impl AsRef<std::path::Path>) {
    let log = std::fs::read_to_string(path).unwrap();
    let log = log.to_lowercase();
    assert!(!log.contains("undecided"));
    assert!(!log.contains("test_failed"));
}

#[test]
fn dummy() {
    let ip_ports = parse_servers_list();
    let _servers = ip_ports
        .iter()
        .map(|(ip, port)| ChildGuard {
            child: Command::new("cargo")
                .arg("run")
                .arg("--release")
                .arg("--")
                .arg(ip.to_string())
                .arg(port.to_string())
                .spawn()
                .unwrap(),
        })
        .collect::<Vec<_>>();

    let output = Command::new("java")
        .arg("-jar")
        .arg("./a4_2025_dummy_tests_v1.jar")
        .arg("--servers-list")
        .arg("servers_list.txt")
        .output()
        .expect("failed to execute process");
    println!("status: {:?}", output);

    check_log_file("A4-dummy.log");
}

#[test]
fn basic() {
    let ip_ports = parse_servers_list();
    let _servers = ip_ports
        .iter()
        .map(|(ip, port)| ChildGuard {
            child: Command::new("cargo")
                .arg("run")
                .arg("--release")
                .arg("--")
                .arg(ip.to_string())
                .arg(port.to_string())
                .spawn()
                .unwrap(),
        })
        .collect::<Vec<_>>();

    let output = Command::new("java")
        .arg("-jar")
        .arg("./a4_2025_basic_tests_v1.jar")
        .arg("--servers-list")
        .arg("servers_list.txt")
        .output()
        .expect("failed to execute process");
    println!("status: {:?}", output);

    check_log_file("A4-basic.log");
}
