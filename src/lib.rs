use ctor::ctor;
use libc::{
    IP_TOS, IPPROTO_IP, RTLD_NEXT, c_char, c_int, c_void, dlerror, dlsym, setsockopt, sockaddr,
    socklen_t,
};
use std::env;
use std::sync::LazyLock;

#[unsafe(no_mangle)]
pub extern "C" fn connect(socket: c_int, address: *const sockaddr, len: socklen_t) -> c_int {
    apply_dscp(socket, *DSCP_CLASS);
    println!(
        "libdscp: moved connection {} to DSCP class {}",
        socket, *DSCP_CLASS
    );
    unsafe { ORIGINAL_CONNECT.unwrap()(socket, address, len) }
}

#[unsafe(no_mangle)]
pub extern "C" fn listen(socket: c_int, backlog: c_int) -> c_int {
    apply_dscp(socket, *DSCP_CLASS);
    println!(
        "libdscp: moved listener {} to DSCP class {}",
        socket, *DSCP_CLASS
    );
    unsafe { ORIGINAL_LISTEN.unwrap()(socket, backlog) }
}

static DSCP_CLASS: LazyLock<u8> = LazyLock::new(|| get_dscp_class());
static mut ORIGINAL_CONNECT: Option<
    unsafe fn(socket: c_int, address: *const sockaddr, len: socklen_t) -> c_int,
> = None;
static mut ORIGINAL_LISTEN: Option<unsafe fn(socket: c_int, backlog: c_int) -> c_int> = None;

#[ctor(unsafe)]
fn init_lib() {
    unsafe {
        ORIGINAL_CONNECT = Some(std::mem::transmute(dlsym_next("connect\0")));
        ORIGINAL_LISTEN = Some(std::mem::transmute(dlsym_next("listen\0")));
    }
}

fn apply_dscp(socket: c_int, dscp: u8) {
    let tos = dscp_to_tos(dscp);
    let tos = c_int::from(tos);
    let socket_res: i32;
    unsafe {
        socket_res = setsockopt(
            socket,
            IPPROTO_IP,
            IP_TOS,
            &tos as *const c_int as *const c_void,
            std::mem::size_of::<c_int>() as socklen_t,
        );
    }
    if socket_res < 0 {
        println!(
            "libdscp: failed to set DSCP for socket {}: {}",
            socket,
            errno::errno(),
        );
    }
}

fn dscp_to_tos(dscp: u8) -> u8 {
    dscp << 2
}

fn get_dscp_class() -> u8 {
    env::var("LIBDSCP_CLASS")
        .map_or_else(|_| Some(0), |var| var.parse::<u8>().ok())
        .unwrap_or_default()
}

fn dlsym_next(symbol: &'static str) -> *const usize {
    unsafe {
        let ptr = dlsym(RTLD_NEXT, symbol.as_ptr() as *const c_char);
        if ptr.is_null() {
            let err = dlerror();
            if err.is_null() {
                panic!(
                    "libdscp: unable to find underlying function for {}: [NO ERROR PRESENT]",
                    symbol
                )
            }

            let err = std::ffi::CStr::from_ptr(err).to_string_lossy().to_string();
            panic!(
                "libdscp: unable to find underlying function for {}: {}",
                symbol, err
            );
        }
        ptr as *const usize
    }
}
