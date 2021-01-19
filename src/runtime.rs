use nix::sched;
use nix::sys::signal::Signal;
use nix::unistd;
use std::process::Command;

use crate::cgroup;
use crate::filesystem;
use crate::mount;
use crate::namespace;

pub struct Runtime<'a> {
	hostname: &'a str,
	rootfs: &'a str,
	cmd: &'a str,
	quota: &'a str,
	mntsrc: &'a str,
	mnttar: &'a str,
	args: Vec<&'a str>
}

fn set_hostname(hostname: &str) {
	// can also use libc here
	unistd::sethostname(hostname).unwrap()
}

impl Runtime<'_> {
	pub fn new<'a>(hostname: &'a str, rootfs: &'a str, cmd: &'a str, quota: &'a str, mntsrc: &'a str, mnttar: &'a str, args: Vec<&'a str>) -> Runtime<'a> {
		Runtime{hostname: &hostname, rootfs: &rootfs, cmd: &cmd, quota: &quota, mntsrc: &mntsrc, mnttar: &mnttar, args}
	}

	#[allow(dead_code)]
	pub fn print_info(&self) {
		println!("{}", self.hostname);
		println!("{}", self.rootfs);
		println!("{}", self.cmd);
		println!("{}", self.quota);
		println!("{}", self.mntsrc);
	}

	fn spawn_child(&self) -> isize {
		/* Set Namespace */
		let group_name = &(self.hostname.to_owned() + "-container");
		namespace::create_isolated_namespace();

		/* Set CGroups */
		cgroup::cgroup_init(group_name);
		if self.quota != "-1" {
			for k in self.quota.split("::") {
				let param: Vec<&str> = k.split(":").collect();
				cgroup::cgroup_quota(param[0], param[1], param[2], group_name);
			}
		}

		set_hostname(self.hostname);
		if self.mntsrc != "-1" {
			mount::mount_tran(self.mntsrc, &(self.rootfs.to_owned() + self.mnttar));
		}

		filesystem::set_root_fs(self.rootfs);
		mount::mount_perm("proc");
		let arg_slice = self.args.as_slice();
		Command::new(self.cmd).args(arg_slice).spawn().expect("Failed to execute container command").wait().unwrap();
		
		mount::unmount_item("proc");
		return 0;
	}
	
	pub fn run_container(&self) {
		const STACK_SIZE: usize = 1024 * 1024;
		let stack: &mut [u8; STACK_SIZE] = &mut [0; STACK_SIZE];
		
		let cb = Box::new(|| self.spawn_child());
	
		let clone_flags = sched::CloneFlags::CLONE_NEWNS | sched::CloneFlags::CLONE_NEWPID | sched::CloneFlags::CLONE_NEWCGROUP | sched::CloneFlags::CLONE_NEWUTS | sched::CloneFlags::CLONE_NEWIPC | sched::CloneFlags::CLONE_NEWNET;
		let _child_pid = sched::clone(cb, stack, clone_flags, Some(Signal::SIGCHLD as i32)).expect("Failed to create child process");
		let _proc_res = nix::sys::wait::waitpid(_child_pid, None);
	}
}