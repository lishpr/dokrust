use nix::mount;

pub fn mount_perm(f: &str) {
	const NONE: Option<&'static [u8]> = None;
	mount::mount(Some(f), f, Some(f), mount::MsFlags::empty(), NONE).expect(&("Failed to mount permanently: ".to_owned() + f));
}

#[allow(dead_code)]
pub fn mount_tran(src: &str, tar: &str) {
	const NONE: Option<&'static [u8]> = None;
	mount::mount(Some(src), tar, NONE, mount::MsFlags::MS_BIND, NONE).expect(&("Failed to mount transiently: ".to_owned() + src));
}

#[allow(dead_code)]
pub fn unmount_item(f: &str) {
	mount::umount(f).unwrap();
}
