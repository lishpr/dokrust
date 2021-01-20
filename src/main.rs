extern crate getopts;
extern crate simple_error;
use getopts::Options;
use simple_error::require_with;
use std::cmp::min;
use std::env;
use std::error::Error;
use std::process;

mod runtime;
mod cgroup;
mod filesystem;
mod mount;
mod namespace;
mod network;

fn print_usage(program: &str, opts: &Options) {
	let brief = format!("Usage: {} [options] [-- <command> <argument>...]", program);
	print!("{}", opts.usage(&brief));
}

fn main() {
	let args: Vec<String> = env::args().collect();
	let program = args[0].clone();
	let mut opts = Options::new();
	opts.optopt("r", "rootfs", "Path to root file-system \ndefault: --rootfs ../rootfs", "");
	opts.optopt("c", "command", "Command to be executed \neg. --command `curl http://google.com`", "");
	opts.optopt("n", "hostname", "Customize the name of your container \ndefault: --hostname dokka", "");
	opts.optopt("q", "quota", "The quota of CGroup for your process \neg. --quota cpu:cpu.cfs_quota_us:50000", "");
	opts.optopt("m", "mount", "Mount directory to container \neg. --mount /root:/mnt", "");
	opts.optopt("k", "network", "Add the hostname of the container that you wish to set the network up for \n eg. --network dokka", "");
	opts.optflag("h", "help", "Print this help menu");


	// Find the conventional "--" that separates out remaing arguments.
	let end_processable = args.iter().position(|s| s == "--").unwrap_or_else(|| args.len());
	let begin_unprocessable = min(end_processable + 1, args.len());

	let matches = opts.parse(&args[1..end_processable]).ok().unwrap_or_else(|| {
		println!("Error: Unrecognzied options");
		print_usage(&program, &opts);
		process::exit(7);
	});

	// Exits early, but doesn't lead to non-zero exit.
	if matches.opt_present("h") {
		print_usage(&program, &opts);
		return;
	}

	let rootfs = if matches.opt_present("r") {matches.opt_str("r").unwrap()} else {String::from("../images/rootfs")};
	let mnt = if matches.opt_present("m") {matches.opt_str("m").unwrap()} else {String::from("-1")};
	let name = if matches.opt_present("n") {matches.opt_str("n").unwrap()} else {String::from("dokka")};
	if matches.opt_present("k") {
		let check = matches.opt_str("k").unwrap_or("dokka-container".to_owned());
		network::net_main(&check);
		return;
	};
	let quota = match matches.opt_str("quota") {
        Some(s) => s,
        None => String::from("-1"),
    };


	let c = matches.opt_str("c"); // NB: Seperate let binding for lifetime.
	let (command, args) = determine_command_tuple(&c, &args[begin_unprocessable..args.len()]).ok().unwrap_or_else(|| {
		println!("Error: Please pass `--command <shell command>` or `-- <command> <argument>...`");
		print_usage(&program, &opts);
		process::exit(7);
	});

	let a: &str;
	let b: &str;
	if mnt != "-1" {
		let param: Vec<&str> = mnt.split(":").collect();
		a = param[0];
		b = param[1];
	} else {
		a = "-1";
		b = "-1";
	}

	let cylinder = runtime::Runtime::new(&name, &rootfs, &command, &quota, a, b, args);
	cylinder.run_container();
}

/// Determines based on the inputs whether we are going to invoke a shell, with
/// shell interpretation, or a simple unescaped argument vector.
/// Rearranges arguments as needed, but doesn't reallocate.
fn determine_command_tuple<'a, T: AsRef<str> + 'a>(shell_command: &'a Option<T>, argv: &'a [T]) -> Result<(&'a str, Vec<&'a str>), Box<dyn Error>> {
	let mut vec: Vec<&str> = vec![];

	// Prepend shell and command string if a command string is given.
	if let Some(shell_command) = shell_command {
		vec.push("/bin/sh");
		vec.push("-c");
		vec.push(shell_command.as_ref());
	}

	// The args given will be the whole command if there's no shell string;
	// otherwise, they'll be added to the argument vector.
	vec.extend(argv.iter().map(|item| item.as_ref()));

	// Shift off the first word as the command, erroring out if no command is
	// given.
	vec.reverse();
	let command = require_with!(vec.pop(), "Empty command!");
	vec.reverse();

	return Ok((command, vec));
}
