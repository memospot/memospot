//! TCP port picker
//!
//! This is a tweaked and slimmed down version of the
//! [portpicker](https://github.com/Dentosal/portpicker-rs) crate.
//!

use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, ToSocketAddrs};
pub type Port = u16;

#[cfg(feature = "rand")]
use {rand::prelude::*, std::ops::Range};

#[cfg(feature = "rand")]
/// Range for random port selection.
static RANDOM_PORT_RANGE: Range<u16> = 15000..45000;

#[cfg(feature = "rand")]
/// Maximum number of retries for random port selection.
static MAX_RANDOM_PORT_RETRIES: u8 = 10;

/// Upper bound for server port.
///
/// Running Memospot in dev mode adds 1 to last used port,
/// so we need to make extra room for that, just in case.
static UPPER_PORT: u16 = 65534;

/// Probe address for checking if a port is free.
///
/// Listening on `0.0.0.0` triggers Windows Firewall, and
/// it shows a pop-up, so we use `127.0.0.1` instead.
static PROBE_ADDR: Ipv4Addr = Ipv4Addr::LOCALHOST;

/// Try to bind to a socket using TCP.
fn test_bind_tcp<A: ToSocketAddrs>(addr: A) -> Option<Port> {
    Some(TcpListener::bind(addr).ok()?.local_addr().ok()?.port())
}

/// Check if a port is free on TCP
pub fn is_free_tcp(port: Port) -> bool {
    let addr = SocketAddrV4::new(PROBE_ADDR, port);
    test_bind_tcp(addr).is_some()
}

/// Request a free port from the OS.
///
/// This works by trying to bind to port 0.
pub fn get_free_tcp() -> Option<Port> {
    let addr = SocketAddrV4::new(PROBE_ADDR, 0);
    test_bind_tcp(addr)
}

#[cfg(feature = "rand")]
/// Get a random port between specified range.
pub fn get_random_free_port(range: Range<u16>, retries: u8) -> Option<Port> {
    let mut rng = rand::rng();
    for _ in 0..retries {
        let port = rng.random_range(range.clone());
        if port < UPPER_PORT && is_free_tcp(port) {
            return Some(port);
        }
    }
    None
}

/// Find a free port.
///
/// Probes the preferred port first, then ask the OS for a free port.
pub fn find_free_port(preferred_port: Port) -> Option<Port> {
    if preferred_port != 0 && preferred_port < UPPER_PORT && is_free_tcp(preferred_port) {
        return Some(preferred_port);
    }

    // Ask the OS for a port
    for _ in 0..10 {
        let port = get_free_tcp()?;
        if port != 0 && port < UPPER_PORT {
            return Some(port);
        }
    }

    // Fall back to random port
    #[cfg(feature = "rand")]
    return get_random_free_port(RANDOM_PORT_RANGE.to_owned(), MAX_RANDOM_PORT_RETRIES);

    #[cfg(not(feature = "rand"))]
    None
}
