use std::fs;
use std::path::PathBuf;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

static CGROUP_PATH: &str = "/sys/fs/cgroup/pids";
static PROC_PATH: &str = "/proc";
static VAR_RUN_PATH: &str = "/var/run/netns";

pub fn net_main(group_name: &str) {
    let pid = fgetpid(group_name);
    link_netns(pid.as_str());
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
    Command::new("ln").arg("-s").arg("-f").arg(src_path.to_str().unwrap()).arg(tar_path.to_str().unwrap()).spawn().unwrap();
}