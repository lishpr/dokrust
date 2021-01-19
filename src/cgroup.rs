use std::fs;
use std::path::PathBuf;

use std::process;
use nix::unistd;

use std::os::unix::fs::PermissionsExt;

static CGROUP_PATH: &str = "/sys/fs/cgroup";

pub fn cgroup_init(group_name: &str) {
	let mut cgroups_path = PathBuf::from(CGROUP_PATH);
	if !cgroups_path.exists() {
		println!("Error: Missing Cgroups Support");
		process::exit(0);
	}
	
	cgroups_path.push("pids");
	cgroups_path.push(group_name);
	if !cgroups_path.exists() {
		fs::create_dir_all(&cgroups_path).unwrap();
		let mut permission = fs::metadata(&cgroups_path).unwrap().permissions();
    	permission.set_mode(0o777);
		fs::set_permissions(&cgroups_path, permission).ok();
	}

	let pids_max = cgroups_path.join("pids.max");
	let notify_on_release = cgroups_path.join("notify_on_release");
	let procs = cgroups_path.join("cgroup.procs");
	
	fs::write(pids_max, b"20").unwrap();
	fs::write(notify_on_release, b"1").unwrap();
	let s: String = unistd::getpid().as_raw().to_string();
    fs::write(procs, format!("{}", s)).unwrap();
}

pub fn cgroup_quota(asset: &str, target: &str, quota: &str, group_name: &str) {	
	// Must be called after the cgroups are initialized!

	let mut cgroups_path = PathBuf::from(CGROUP_PATH);

	cgroups_path.push(asset);
	cgroups_path.push(group_name);
	if !cgroups_path.exists() {
		fs::create_dir_all(&cgroups_path).unwrap();
		let mut permission = fs::metadata(&cgroups_path).unwrap().permissions();
    	permission.set_mode(0o777);
		fs::set_permissions(&cgroups_path, permission).ok();
	}

	let tasks = cgroups_path.join("tasks");
	let cfs_quota = cgroups_path.join(target);
	
	fs::write(cfs_quota, quota.as_bytes()).unwrap();
    fs::write(tasks, format!("{}", unistd::getpid().as_raw())).unwrap();
}

#[allow(dead_code)]
pub fn cgroup_deinit(group_name: &str){
	let mut cgroups_path = PathBuf::from(CGROUP_PATH);
	if !cgroups_path.exists() {
		println!("Error: Missing Cgroups Support");
		process::exit(0);
	}
	cgroups_path.push(group_name);
	fs::remove_dir(cgroups_path).expect("Failed to remove the cgroup");
}

