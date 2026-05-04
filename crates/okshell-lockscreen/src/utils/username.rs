pub fn current_username() -> String {
    unsafe {
        let uid = libc::getuid();
        let pw = libc::getpwuid(uid);
        std::ffi::CStr::from_ptr((*pw).pw_name)
            .to_string_lossy()
            .into_owned()
    }
}
