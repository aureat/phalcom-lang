//! Single-VM-thread descriptor readiness poller backed by `mio`.
//!
//! # Law of Confinement (PDR-0016)
//! No `mio` type appears outside `phalcom-core/src/reactor/`.
//! External layers only interact with reactor tokens and plain results.

use super::registry::ReactorToken;
use std::io;
use std::sync::Arc;
use std::time::Duration;

/// Reserved sentinel token for cross-thread wakers.
pub const WAKER_TOKEN: mio::Token = mio::Token(usize::MAX);

/// Desired I/O interest for a pollable descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PollInterest {
    /// Interest in read readiness.
    Readable,
    /// Interest in write readiness.
    Writable,
    /// Interest in both read and write readiness.
    Both,
}

impl From<PollInterest> for mio::Interest {
    fn from(interest: PollInterest) -> Self {
        match interest {
            PollInterest::Readable => mio::Interest::READABLE,
            PollInterest::Writable => mio::Interest::WRITABLE,
            PollInterest::Both => mio::Interest::READABLE | mio::Interest::WRITABLE,
        }
    }
}

/// Observed readiness events on a descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PollReadiness {
    /// Descriptor has data ready to read.
    pub is_readable: bool,
    /// Descriptor is ready to accept writes.
    pub is_writable: bool,
    /// Remote read end closed (hangup / EOF).
    pub is_read_closed: bool,
    /// Remote write end closed.
    pub is_write_closed: bool,
    /// Error condition on the descriptor.
    pub is_error: bool,
}

impl PollReadiness {
    /// Creates a [`PollReadiness`] from a `mio` event.
    pub fn from_mio_event(event: &mio::event::Event) -> Self {
        Self {
            is_readable: event.is_readable(),
            is_writable: event.is_writable(),
            is_read_closed: event.is_read_closed(),
            is_write_closed: event.is_write_closed(),
            is_error: event.is_error(),
        }
    }

    /// Returns `true` if any readiness flag is set.
    pub fn is_ready(&self) -> bool {
        self.is_readable || self.is_writable || self.is_read_closed || self.is_write_closed || self.is_error
    }

    /// Encodes readiness flags into a compact integer bitmask.
    pub fn bits(&self) -> u32 {
        let mut bits = 0u32;
        if self.is_readable {
            bits |= 1 << 0;
        }
        if self.is_writable {
            bits |= 1 << 1;
        }
        if self.is_read_closed {
            bits |= 1 << 2;
        }
        if self.is_write_closed {
            bits |= 1 << 3;
        }
        if self.is_error {
            bits |= 1 << 4;
        }
        bits
    }
}

/// The reactor's OS poller instance wrapping `mio::Poll` and cross-thread `mio::Waker`.
pub struct Poller {
    poll: mio::Poll,
    events: mio::Events,
    waker: Arc<mio::Waker>,
}

impl Poller {
    /// Creates a new poller instance and registers a cross-thread waker.
    pub fn new() -> io::Result<Self> {
        let poll = mio::Poll::new()?;
        let waker = Arc::new(mio::Waker::new(poll.registry(), WAKER_TOKEN)?);
        let events = mio::Events::with_capacity(128);

        Ok(Self { poll, events, waker })
    }

    /// Returns an `Arc<mio::Waker>` handle that can be used from off-thread workers
    /// to awaken the poller parked on the VM thread.
    pub fn waker(&self) -> Arc<mio::Waker> {
        Arc::clone(&self.waker)
    }

    /// Registers interest on an I/O source using the given `ReactorToken`'s slot index.
    pub fn register<S: mio::event::Source + ?Sized>(&mut self, source: &mut S, token: ReactorToken, interest: PollInterest) -> io::Result<()> {
        self.poll.registry().register(source, mio::Token(token.slot as usize), interest.into())
    }

    /// Re-arms/updates interest on an already registered I/O source.
    pub fn reregister<S: mio::event::Source + ?Sized>(&mut self, source: &mut S, token: ReactorToken, interest: PollInterest) -> io::Result<()> {
        self.poll.registry().reregister(source, mio::Token(token.slot as usize), interest.into())
    }

    /// Deregisters an I/O source from the poller.
    pub fn deregister<S: mio::event::Source + ?Sized>(&mut self, source: &mut S) -> io::Result<()> {
        self.poll.registry().deregister(source)
    }

    /// Registers a raw Unix file descriptor with the poller.
    #[cfg(unix)]
    pub fn register_raw_fd(&mut self, fd: std::os::unix::io::RawFd, token: ReactorToken, interest: PollInterest) -> io::Result<()> {
        let mut source = mio::unix::SourceFd(&fd);
        self.register(&mut source, token, interest)
    }

    /// Re-arms a raw Unix file descriptor with the poller.
    #[cfg(unix)]
    pub fn reregister_raw_fd(&mut self, fd: std::os::unix::io::RawFd, token: ReactorToken, interest: PollInterest) -> io::Result<()> {
        let mut source = mio::unix::SourceFd(&fd);
        self.reregister(&mut source, token, interest)
    }

    /// Deregisters a raw Unix file descriptor from the poller.
    #[cfg(unix)]
    pub fn deregister_raw_fd(&mut self, fd: std::os::unix::io::RawFd) -> io::Result<()> {
        let mut source = mio::unix::SourceFd(&fd);
        self.deregister(&mut source)
    }

    /// Waits for descriptor readiness or waker events up to `timeout`.
    ///
    /// Returns a list of `(slot_index, PollReadiness)` events observed during the poll.
    /// Waker events are consumed internally without generating descriptor events.
    pub fn poll(&mut self, timeout: Option<Duration>) -> io::Result<Vec<(u32, PollReadiness)>> {
        self.poll.poll(&mut self.events, timeout)?;

        let mut ready_events = Vec::new();
        for event in self.events.iter() {
            if event.token() == WAKER_TOKEN {
                continue;
            }
            let slot = event.token().0 as u32;
            let readiness = PollReadiness::from_mio_event(event);
            ready_events.push((slot, readiness));
        }
        self.events.clear();
        Ok(ready_events)
    }

    /// Unblocks any active poll call on the VM thread.
    pub fn wake(&self) -> io::Result<()> {
        self.waker.wake()
    }
}
