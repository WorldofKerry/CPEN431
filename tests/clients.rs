use std::{fs::File, net::Ipv4Addr, process::Command};
use tempfile::NamedTempFile;
use std::io::Write;

struct ChildGuard {
    pub child: std::process::Child,
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        self.child.kill().unwrap();
    }
}

fn create_ips_ports(count: usize) -> (NamedTempFile, Vec<(Ipv4Addr, u16)>) {
    let mut file = NamedTempFile::new().unwrap();
    let ips_ports = (0..count)
        .map(|_| {
            let ip = Ipv4Addr::LOCALHOST;
            let port = portpicker::pick_unused_port().unwrap();
            writeln!(file, "{}:{}", ip, port).unwrap();
            (ip, port)
        })
        .collect();
    dbg!(file, ips_ports)
}

fn check_log_file(path: impl AsRef<std::path::Path>) {
    let log = std::fs::read_to_string(path).unwrap();
    let log = log.to_lowercase();
    assert!(!log.contains("undecided"));
    assert!(!log.contains("test_failed"));
    assert!(!log.contains("FAIL_IMMEDIATELY"));
}

fn client_runner(name: &str) {
    let (file, ip_ports) = create_ips_ports(1);
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
    Command::new("java")
        .arg("-jar")
        .arg(format!("./a4_2025_{name}_tests_v1.jar"))
        .arg("--servers-list")
        .arg(file.path())
        .output()
        .expect("failed to execute process");
    check_log_file(format!("A4-{name}.log"));
}

#[test]
fn dummy() {
    client_runner("dummy");
}

#[test]
fn basic() {
    client_runner("basic");
}
