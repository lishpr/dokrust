use std::fs;
use std::path::PathBuf;

static CGROUP_PATH: &str = "/sys/fs/cgroup";

pub fn fgetpid(group_name: &str) {
    let mut cgroups_path = PathBuf::from(CGROUP_PATH);
    cgroups_path.push("pids");
    cgroups_path.push(group_name);
    if !cgroups_path.exists() {
        return;
    }

    let procs = cgroups_path.join("cgroup.procs");
    let s = String::from_utf8(fs::read(procs).unwrap()).unwrap();
    let mut ss = s.split_whitespace();
    ss.next();
    println!("{}", ss.next().unwrap());
    // return String::from_utf8(fs::read(procs)?).into_string();
}