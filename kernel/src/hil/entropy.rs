//! Interfaces for accessing an entropy source.
//!
//! An entropy source produces random bits that are computationally
//! intractable to guess, even if the complete history of generated
//! bits and state of the device can be observed. Entropy sources must
//! generate bits from an underlying physical random process, such as
//! thermal noise, radiation, avalanche noise, or circuit
//! instability. These bits of entropy can be used to seed
//! cryptographically strong random number generators. Because high-quality
//! entropy is critical for security and these APIs provide entropy,
//! it is important to understand all of these requirements before
//! implementing these traits. Otherwise you may subvert the security
//! of the operating system.
//!
//! _Entropy_: Entropy bits generated by this trait MUST have very high
//! entropy, i.e. 1 bit of entropy per generated bit. If the underlying
//! source generates <1 bit of entropy per bit, these low-entropy bits
//! SHOULD be mixed and combined with a cryptographic hash function.
//! A good, short reference on the difference between entropy and
//! randomness as well as guidelines for high-entropy sources is
//! Recommendations for Randomness in the Operating System: How to
//! Keep Evil Children Out of Your Pool and Other Random Facts,
//! Corrigan-Gibbs et al., HotOS 2015.
//!
//! The interface is designed to work well with entropy generators
//! that may not have values ready immediately. This is important when
//! generating numbers from a low-bandwidth hardware entropy source
//! generator or when virtualized among many consumers.
//!
//! Entropy is yielded to a Client as an `Iterator` which only
//! terminates when no more entropy is currently available. Clients
//! can request more entropy if needed and will be called again when
//! more is available.
//!
//! # Example
//!
//! The following example is a simple capsule that prints out entropy
//! once a second using the `Alarm` and `Entropy` traits.
//!
//! ```
//! use kernel::hil;
//! use kernel::hil::entropy::Entropy32;
//! use kernel::hil::entropy::Client32;
//! use kernel::hil::time::Alarm;
//! use kernel::hil::time::Frequency;
//! use kernel::hil::time::Client;
//! use kernel::ReturnCode;
//!
//! struct EntropyTest<'a, A: 'a + Alarm> {
//!     entropy: &'a Entropy32 <'a>,
//!     alarm: &'a A
//! }
//!
//! impl<'a, A: Alarm> EntropyTest<'a, A> {
//!     pub fn initialize(&self) {
//!         let interval = 1 * <A::Frequency>::frequency();
//!         let tics = self.alarm.now().wrapping_add(interval);
//!         self.alarm.set_alarm(tics);
//!     }
//! }
//!
//! impl<'a, A: Alarm> Client for EntropyTest<'a, A> {
//!     fn fired(&self) {
//!         self.entropy.get();
//!     }
//! }
//!
//! impl<'a, A: Alarm> Client32 for EntropyTest<'a, A> {
//!     fn entropy_available(&self,
//!                          entropy: &mut Iterator<Item = u32>,
//!                          error: ReturnCode) -> hil::entropy::Continue {
//!         match entropy.next() {
//!             Some(val) => {
//!                 println!("Entropy {}", val);
//!                 let interval = 1 * <A::Frequency>::frequency();
//!                 let tics = self.alarm.now().wrapping_add(interval);
//!                 self.alarm.set_alarm(tics);
//!                 hil::entropy::Continue::Done
//!             },
//!             None => hil::entropy::Continue::More
//!         }
//!     }
//! }
//! ```

use returncode::ReturnCode;
/// Denotes whether the [Client](trait.Client.html) wants to be notified when
/// `More` randomness is available or if they are `Done`
#[derive(Debug, Eq, PartialEq)]
pub enum Continue {
    /// More randomness is required.
    More,
    /// No more randomness required.
    Done,
}

/// Generic interface for a 32-bit entropy source.
///
/// Implementors should assume the client implements the
/// [Client](trait.Client32.html) trait.
pub trait Entropy32<'a> {
    /// Initiate the aquisition of entropy.
    ///
    /// There are three valid return values:
    ///   - SUCCESS: a `entropy_available` callback will be called in
    ///     the future when entropy is available.
    ///   - FAIL: a `entropy_available` callback will not be called in
    ///     the future, because entropy cannot be generated. This
    ///     is a general failure condition.
    ///   - EOFF: a `entropy_available` callback will not be called in
    ///     the future, because the random number generator is off/not
    ///     powered.
    fn get(&self) -> ReturnCode;

    /// Cancel acquisition of entropy.
    ///
    /// There are three valid return values:
    ///   - SUCCESS: an outstanding request from `get` has been cancelled,
    ///     or there was no outstanding request. No `entropy_available`
    ///     callback will be issued.
    ///   - FAIL: There will be a `entropy_available` callback, which
    ///     may or may not return an error code.
    fn cancel(&self) -> ReturnCode;

    /// Set the client to receive `entropy_available` callbacks.
    fn set_client(&'a self, &'a Client32);
}

/// An [Entropy32](trait.Entropy32.html) client
///
/// Clients of an [Entropy32](trait.Entropy32.html) must implement this trait.
pub trait Client32 {
    /// Called by the (Entropy)[trait.Entropy32.html] when there is entropy
    /// available.
    ///
    /// `entropy` in an `Iterator` of available entropy. The amount of
    /// entropy available may increase if `entropy` is not consumed
    /// quickly so clients should not rely on iterator termination to
    /// finish consuming entropy.
    ///
    /// The client returns either `Continue::More` if the iterator did
    /// not have enough entropy (indicating another
    /// `entropy_available` callback is requested) and the client
    /// would like to be called again when more is available, or
    /// `Continue::Done`, which indicates `entropy_available` should
    /// not be called again until `get()` is called.
    ///
    /// If `entropy_available` is triggered after a call to `cancel()`
    /// then error MUST be ECANCEL and `entropy` MAY contain bits of
    /// entropy.
    fn entropy_available(&self, entropy: &mut Iterator<Item = u32>, error: ReturnCode) -> Continue;
}

/// An 8-bit entropy generator.
///
/// Implementors should assume the client implements the
/// [Client8](trait.Client8.html) trait.
pub trait Entropy8<'a> {
    /// Initiate the acquisition of new entropy.
    ///
    /// There are three valid return values:
    ///   - SUCCESS: a `entropy_available` callback will be called in
    ///     the future when entropy is available.
    ///   - FAIL: a `entropy_available` callback will not be called in
    ///     the future, because entropy cannot be generated. This
    ///     is a general failure condition.
    ///   - EOFF: a `entropy_available` callback will not be called in
    ///     the future, because the entropy generator is off/not
    ///     powered.
    fn get(&self) -> ReturnCode;

    /// Cancel acquisition of entropy.
    ///
    /// There are three valid return values:
    ///   - SUCCESS: an outstanding request from `get` has been cancelled,
    ///     or there was no outstanding request. No `entropy_available`
    ///     callback will be issued.
    ///   - FAIL:: There will be a `entropy_available` callback, which
    ///     may or may not return an error code.
    fn cancel(&self) -> ReturnCode;

    /// Set the client to receive `entropy_available` callbacks.
    fn set_client(&'a self, &'a Client8);
}

/// An [Entropy8](trait.Entropy8.html) client
///
/// Clients of an [Entropy8](trait.Entropy8.html) must implement this trait.
pub trait Client8 {
    /// Called by the (Entropy)[trait.Entropy8.html] when there are
    /// one or more bytes of entropy available.
    ///
    /// `entropy` in an `Iterator` of available entropy. The amount of
    /// entropy available may increase if `entropy` is not consumed
    /// quickly so clients should not rely on iterator termination to
    /// finish consuming entropy.
    ///
    /// The client returns either `Continue::More` if the iterator did
    /// not have enough entropy (indicating another
    /// `entropy_available` callback is requested) and the client
    /// would like to be called again when more is available, or
    /// `Continue::Done`, which indicates `entropy_available` should
    /// not be called again until `get()` is called.
    ///
    /// If `entropy_available` is triggered after a call to `cancel()`
    /// then error MUST be ECANCEL and `entropy` MAY contain bits of
    /// entropy.
    fn entropy_available(&self, entropy: &mut Iterator<Item = u8>, error: ReturnCode) -> Continue;
}
