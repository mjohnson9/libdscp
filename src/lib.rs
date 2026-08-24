use ctor::ctor;
use libc::{
    AF_INET, AF_INET6, IP_TOS, IPPROTO_IP, IPPROTO_IPV6, IPV6_TCLASS, RTLD_NEXT, SO_DOMAIN,
    SOL_SOCKET, c_char, c_int, c_void, dlerror, dlsym, getsockopt, setsockopt, sockaddr, socklen_t,
};
use std::env;
use std::ffi::CStr;
use std::sync::LazyLock;

/// Passthrough a connection attempt to the libc `connect`. See `connect(2)` documentation for more information.
///
/// # Safety
///
/// - This function is `unsafe` because it is a wrapper around a libc function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn connect(socket: c_int, address: *const sockaddr, len: socklen_t) -> c_int {
    apply_dscp(socket, *DSCP_CLASS);
    unsafe { (*ORIGINAL_CONNECT)(socket, address, len) }
}

/// Passthrough a listener to the libc `listen`. See `listen(2)` documentation for more information.
///
/// # Safety
///
/// - This function is `unsafe` because it is a wrapper around a libc function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn listen(socket: c_int, backlog: c_int) -> c_int {
    apply_dscp(socket, *DSCP_CLASS);
    unsafe { (*ORIGINAL_LISTEN)(socket, backlog) }
}

static DSCP_CLASS: LazyLock<u8> = LazyLock::new(get_dscp_class);
static IS_DEBUG: LazyLock<bool> = LazyLock::new(get_debug);
static ORIGINAL_CONNECT: LazyLock<
    unsafe extern "C" fn(socket: c_int, address: *const sockaddr, len: socklen_t) -> c_int,
> = LazyLock::new(|| unsafe { std::mem::transmute(dlsym_next(c"connect")) });
static ORIGINAL_LISTEN: LazyLock<unsafe extern "C" fn(socket: c_int, backlog: c_int) -> c_int> =
    LazyLock::new(|| unsafe { std::mem::transmute(dlsym_next(c"listen")) });

#[ctor(unsafe)]
fn init_lib() {
    // Preload all of the LazyLocks
    let _ = *IS_DEBUG;
    let _ = *DSCP_CLASS;
    let _ = *ORIGINAL_CONNECT;
    let _ = *ORIGINAL_LISTEN;
}

fn apply_dscp(socket: c_int, dscp: u8) {
    let socket_family = get_socket_family(socket);
    let (level, optname) = match socket_family {
        Err(socket_family_error) => {
            eprintln!(
                "libdscp: failed to get socket type for socket {}: {}",
                socket, socket_family_error,
            );
            return;
        }
        Ok(AF_INET) => (IPPROTO_IP, IP_TOS),
        Ok(AF_INET6) => (IPPROTO_IPV6, IPV6_TCLASS),
        _ => {
            if *IS_DEBUG {
                eprintln!("libdscp: socket {} is not IPv4/IPv6, skipping", socket);
            }
            return;
        }
    };

    let old_tos = get_socket_tos(socket, level, optname);
    if let Err(tos_error) = old_tos {
        eprintln!(
            "libdscp: failed to get socket TOS/TCLASS for socket {}: {}",
            socket, tos_error,
        );
        return;
    }

    let new_tos = dscp_to_tos(old_tos.unwrap().try_into().unwrap_or_default(), dscp);
    let new_tos = c_int::from(new_tos);

    let socket_res = unsafe {
        setsockopt(
            socket,
            level,
            optname,
            &new_tos as *const c_int as *const c_void,
            std::mem::size_of::<c_int>() as socklen_t,
        )
    };
    if socket_res < 0 {
        eprintln!(
            "libdscp: failed to set DSCP for socket {}: {}",
            socket,
            std::io::Error::last_os_error(),
        );
    } else if *IS_DEBUG {
        eprintln!(
            "libdscp: moved socket {} to DSCP class {}",
            socket, *DSCP_CLASS
        );
    }
}

fn get_socket_family(socket: c_int) -> Result<c_int, std::io::Error> {
    let mut family: c_int = 0;
    let mut len: socklen_t = std::mem::size_of::<c_int>().try_into().unwrap();
    let res = unsafe {
        getsockopt(
            socket,
            SOL_SOCKET,
            SO_DOMAIN,
            &mut family as *mut c_int as *mut c_void,
            &mut len as *mut socklen_t,
        )
    };
    if res < 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(family)
}

fn get_socket_tos(socket: c_int, level: c_int, optname: c_int) -> Result<c_int, std::io::Error> {
    let mut tos: c_int = 0;
    let mut len: socklen_t = std::mem::size_of::<c_int>().try_into().unwrap();
    let res = unsafe {
        getsockopt(
            socket,
            level,
            optname,
            &mut tos as *mut c_int as *mut c_void,
            &mut len as *mut socklen_t,
        )
    };
    if res < 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(tos)
}

fn dscp_to_tos(tos: u8, dscp: u8) -> u8 {
    ((dscp & 0b111111) << 2) | (tos & 0b11)
}

fn get_dscp_class() -> u8 {
    let env_var = env::var("LIBDSCP_CLASS");
    match env_var {
        Err(_) => {
            if *IS_DEBUG {
                eprintln!("libdscp: no LIBDSCP_CLASS found; defaulting to 0");
            }
            0
        }
        Ok(v) => {
            let dscp = v.parse();
            match dscp {
                Err(_) => {
                    eprintln!(
                        "libdscp: failed to parse LIBDSCP_CLASS as a number; defaulting to 0"
                    );
                    0
                }
                Ok(dscp) => {
                    if dscp > 63 {
                        eprintln!("libdscp: provided LIBDSCP_CLASS exceeds 63; defaulting to 0");
                        return 0;
                    }
                    dscp
                }
            }
        }
    }
}

fn get_debug() -> bool {
    let debug_int = env::var("LIBDSCP_DEBUG")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    debug_int != 0
}

fn dlsym_next(symbol: &'static CStr) -> *const usize {
    unsafe {
        let ptr = dlsym(RTLD_NEXT, symbol.as_ptr() as *const c_char);
        if ptr.is_null() {
            let err = dlerror();
            if err.is_null() {
                panic!(
                    "libdscp: unable to find underlying function for {:?}: [NO ERROR PRESENT]",
                    symbol
                )
            }

            let err = std::ffi::CStr::from_ptr(err).to_string_lossy().to_string();
            panic!(
                "libdscp: unable to find underlying function for {:?}: {}",
                symbol, err
            );
        }
        ptr as *const usize
    }
}
