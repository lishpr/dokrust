use nix::mount;

pub fn mount_perm(f: &str) {
	const NONE: Option<&'static [u8]> = None;
	mount::mount(Some(f), f, Some(f), mount::MsFlags::empty(), NONE).expect(&("Failed to mount ".to_owned() + f));
}

pub fn unmount_perm(f: &str) {
	mount::umount(f).unwrap();
}