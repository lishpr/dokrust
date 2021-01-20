use std::fs;
use std::path::PathBuf;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

static CGROUP_PATH: &str = "/sys/fs/cgroup/pids";
static PROC_PATH: &str = "/proc";
static VAR_RUN_PATH: &str = "/var/run/netns";
static ETC_NETNS_PATH: &str = "/etc/netns"
static BR0: &str = "br0";

pub fn net_main(group_name: &str) {
    let pid = &fgetpid(group_name);
    link_netns(pid);
    command_set(pid);
    internet_conn();
}

fn internet_conn() {
    Command::new("sysctl").args(&["-w", "net.ipv4.ip_forward=1"]).spawn().expect("1").wait().unwrap();
    Command::new("iptables").args(&["-t", "nat", "-A", "POSTROUTING", "-s", "10.1.1.0/24", "!", "-o", BR0, "-j", "MASQUERADE"]).spawn().expect("1").wait().unwrap();
    let mut br0_path = PathBuf::from(ETC_NETNS_PATH);
    br0_path.push(BR0);
    if !br0_path.exists() {
        fs::create_dir_all(&br0_path).unwrap();
        let mut permission = fs::metadata(&br0_path).unwrap().permissions();
        permission.set_mode(0o777);
        fs::set_permissions(&br0_path, permission).ok();
    }
    let resolv = br0_path.join("resolv.conf");
    fs::write(resolv, b"nameserver 100.100.5.6").unwrap();
}

fn command_set(pid: &str) {
    let veth1 = &("veth1-".to_owned() + pid);
    let veth2 = &("veth2-".to_owned() + pid);
    let con1 = pid;
    Command::new("ip").args(&["link", "add", "name"]).arg(veth1).args(&["type", "veth", "peer", "name"]).arg(veth2).spawn().expect("1").wait().unwrap();
    Command::new("ip").args(&["link", "set"]).arg(veth1).arg("netns").arg(con1).spawn().expect("2").wait().unwrap();
    Command::new("ip").args(&["link", "add", "name", BR0, "type", "bridge"]).spawn().expect("3").wait().unwrap();
    Command::new("ip").args(&["link", "set", veth2, "master", BR0]).spawn().expect("4").wait().unwrap();
    Command::new("ip").args(&["addr", "add", "10.1.1.1/24", "brd", "+", "dev", BR0]).spawn().expect("5").wait().unwrap();
    Command::new("ip").args(&["netns", "exec", con1, "ip", "addr", "add", "10.1.1.1/24", "dev", veth1]).spawn().expect("6").wait().unwrap();
    Command::new("ip").args(&["netns", "exec", con1, "ip", "link", "set", veth1, "up"]).spawn().expect("7").wait().unwrap();
    Command::new("ip").args(&["netns", "exec", con1, "ip", "link", "set", "lo", "up"]).spawn().expect("8").wait().unwrap();
    Command::new("ip").args(&["link", "set", veth2, "up"]).spawn().expect("9").wait().unwrap();
    Command::new("ip").args(&["link", "set", BR0, "up"]).spawn().expect("10").wait().unwrap();
    Command::new("ip").args(&["link", "set", BR0, "up"]).spawn().expect("11").wait().unwrap();
    Command::new("ip").args(&["netns", "exec", con1, "ip", "route", "add", "default", "via", "10.1.1.1"]).spawn().expect("12").wait().unwrap();
}

fn fgetpid(group_name: &str) -> String {
    let mut cgroups_path = PathBuf::from(CGROUP_PATH);
    cgroups_path.push(group_name);
    if !cgroups_path.exists() {
        return "".to_owned();
    }

    let procs = cgroups_path.join("cgroup.procs");
    let s = String::from_utf8(fs::read(procs).unwrap()).unwrap();
    let mut ss = s.split_whitespace();
    ss.next();
    let the_pid = ss.next().unwrap().clone();
    println!("{}", the_pid);
    return the_pid.to_owned();
}

fn link_netns(netns_name: &str) {
    let mut src_path = PathBuf::from(PROC_PATH);
    src_path.push(netns_name);
    src_path.push("ns/net");

    let mut tar_path = PathBuf::from(VAR_RUN_PATH);
    if !tar_path.exists() {
        fs::create_dir_all(&tar_path).unwrap();
        let mut permission = fs::metadata(&tar_path).unwrap().permissions();
        permission.set_mode(0o777);
        fs::set_permissions(&tar_path, permission).ok();
    }

    tar_path.push(netns_name);
    println!("{}", src_path.to_str().unwrap());

    println!("{}", tar_path.to_str().unwrap());
    symlink(src_path.to_str().unwrap(), tar_path.to_str().unwrap());
}

fn symlink(src: &str, des: &str) {
    Command::new("ln").arg("-s").arg("-f").arg(src).arg(des).spawn().expect("1").wait().unwrap();
}