#[cfg(target_os = "macos")]
use std::os::unix::io::AsRawFd;
#[cfg(target_os = "macos")]
use std::os::unix::net::UnixStream;

#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct audit_token_t {
    pub val: [u32; 8],
}

#[cfg(target_os = "macos")]
extern "C" {
    pub fn audit_token_to_pid(atoken: audit_token_t) -> libc::pid_t;
    pub fn audit_token_to_euid(atoken: audit_token_t) -> libc::uid_t;
    pub fn audit_token_to_auid(atoken: audit_token_t) -> libc::uid_t;
}

#[cfg(target_os = "macos")]
pub fn get_audit_token(stream: &UnixStream) -> std::io::Result<audit_token_t> {
    use std::mem;

    let fd = stream.as_raw_fd();
    let mut token: audit_token_t = unsafe { mem::zeroed() };
    let mut token_len = mem::size_of::<audit_token_t>() as libc::socklen_t;

    let res = unsafe {
        libc::getsockopt(
            fd,
            0, // SOL_LOCAL
            6, // LOCAL_PEERTOKEN
            &mut token as *mut _ as *mut libc::c_void,
            &mut token_len,
        )
    };

    if res == 0 {
        Ok(token)
    } else {
        Err(std::io::Error::last_os_error())
    }
}

#[cfg(target_os = "macos")]
pub fn get_pid_from_token(token: audit_token_t) -> libc::pid_t {
    unsafe { audit_token_to_pid(token) }
}

#[cfg(target_os = "macos")]
pub fn get_uid_from_token(token: audit_token_t) -> libc::uid_t {
    unsafe { audit_token_to_euid(token) }
}
