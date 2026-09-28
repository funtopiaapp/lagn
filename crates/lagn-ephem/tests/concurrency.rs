//! Thread-safety and global-state tests.
//!
//! Swiss Ephemeris keeps process-global mutable state: the ephemeris path, the
//! sidereal mode, and a cache of open file handles. `lagn-ephem` claims to
//! serialise all of it behind a mutex. That claim was documented but never
//! exercised, and an untested concurrency guarantee is not a guarantee.
//!
//! The dangerous interleaving is specifically: thread A sets the sidereal mode
//! to Lahiri, thread B sets it to Raman, thread A then reads a position and
//! silently gets Raman values. These tests try hard to produce exactly that.

use std::sync::{Arc, Barrier};
use std::thread;

use lagn_ephem::*;

fn init() {
    let d = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../ephe").canonicalize().unwrap();
    Ephemeris::set_ephemeris_path(d).unwrap();
}

const JD: f64 = 2446237.875;

#[test]
fn concurrent_readers_with_the_same_settings_agree() {
    init();
    let expected = Ephemeris::new(Ayanamsa::Lahiri, NodeType::Mean).positions(JD).unwrap();
    let expected = Arc::new(expected);
    let barrier = Arc::new(Barrier::new(16));

    let handles: Vec<_> = (0..16)
        .map(|_| {
            let expected = Arc::clone(&expected);
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                let e = Ephemeris::new(Ayanamsa::Lahiri, NodeType::Mean);
                barrier.wait();
                for _ in 0..300 {
                    assert_eq!(e.positions(JD).unwrap(), *expected);
                }
            })
        })
        .collect();
    for h in handles {
        h.join().expect("worker panicked");
    }
}

#[test]
fn concurrent_readers_with_different_ayanamsas_do_not_contaminate_each_other() {
    init();
    // Precompute the truth for each ayanamsa serially.
    let truth: Vec<(Ayanamsa, [(Graha, Position); 9])> =
        [Ayanamsa::Lahiri, Ayanamsa::Raman, Ayanamsa::Krishnamurti, Ayanamsa::TrueChitra]
            .into_iter()
            .map(|a| (a, Ephemeris::new(a, NodeType::Mean).positions(JD).unwrap()))
            .collect();
    // They must genuinely differ, or the test proves nothing.
    assert_ne!(truth[0].1[0].1.longitude, truth[1].1[0].1.longitude);

    let truth = Arc::new(truth);
    let barrier = Arc::new(Barrier::new(24));
    let handles: Vec<_> = (0..24)
        .map(|i| {
            let truth = Arc::clone(&truth);
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                let (aya, expected) = truth[i % truth.len()];
                let e = Ephemeris::new(aya, NodeType::Mean);
                barrier.wait();
                for _ in 0..400 {
                    let got = e.positions(JD).unwrap();
                    assert_eq!(got, expected, "{aya:?} was contaminated by another thread");
                }
            })
        })
        .collect();
    for h in handles {
        h.join().expect("worker panicked");
    }
}

#[test]
fn concurrent_mixed_operations_stay_consistent() {
    init();
    // Interleave every kind of call that touches the global sidereal mode.
    let lahiri = Ephemeris::new(Ayanamsa::Lahiri, NodeType::Mean);
    let want_pos = lahiri.position(JD, Graha::Moon).unwrap();
    let want_aya = lahiri.ayanamsa_value(JD).unwrap();
    let (want_ang, _) = lahiri.angles(JD, 13.0827, 80.2707, HouseSystem::WholeSign).unwrap();

    let barrier = Arc::new(Barrier::new(20));
    let handles: Vec<_> = (0..20)
        .map(|i| {
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                let e = Ephemeris::new(Ayanamsa::Lahiri, NodeType::Mean);
                let noise = Ephemeris::new(Ayanamsa::Raman, NodeType::True);
                barrier.wait();
                for k in 0..200 {
                    // Half the threads keep flipping the global mode.
                    if i % 2 == 0 {
                        let _ = noise.positions(JD + k as f64).unwrap();
                        let _ = noise.ayanamsa_value(JD).unwrap();
                    }
                    match k % 3 {
                        0 => assert_eq!(e.position(JD, Graha::Moon).unwrap(), want_pos),
                        1 => assert_eq!(e.ayanamsa_value(JD).unwrap(), want_aya),
                        _ => {
                            let (a, _) = e
                                .angles(JD, 13.0827, 80.2707, HouseSystem::WholeSign)
                                .unwrap();
                            assert_eq!(a, want_ang);
                        }
                    }
                }
            })
        })
        .collect();
    for h in handles {
        h.join().expect("worker panicked");
    }
}

#[test]
fn a_panicking_thread_does_not_poison_the_lock() {
    init();
    // The mutex guard is recovered with `into_inner` on poisoning precisely so
    // that one bad call cannot wedge the whole process.
    let before = Ephemeris::new(Ayanamsa::Lahiri, NodeType::Mean).positions(JD).unwrap();

    let h = thread::spawn(|| {
        let e = Ephemeris::new(Ayanamsa::Lahiri, NodeType::Mean);
        let _ = e.positions(JD).unwrap();
        panic!("deliberate panic while the library has been used");
    });
    assert!(h.join().is_err(), "the thread was supposed to panic");

    // The ephemeris must still be usable and still correct.
    let after = Ephemeris::new(Ayanamsa::Lahiri, NodeType::Mean).positions(JD).unwrap();
    assert_eq!(after, before, "state was corrupted by a panicking thread");
}

#[test]
fn setting_the_ephemeris_path_twice_is_idempotent_but_a_change_is_refused() {
    init();
    let d = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../ephe").canonicalize().unwrap();
    // Same path again: fine.
    assert!(Ephemeris::set_ephemeris_path(&d).is_ok());
    // A different path: refused, because Swiss Ephemeris caches file handles
    // against the first one and would silently keep using them.
    let other = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).canonicalize().unwrap();
    let r = Ephemeris::set_ephemeris_path(other);
    assert!(
        matches!(r, Err(EphemError::EphemerisPathConflict { .. })),
        "expected a path conflict, got {r:?}"
    );
    // A nonexistent path is refused outright.
    assert!(Ephemeris::set_ephemeris_path("/no/such/directory/anywhere").is_err());
}

#[test]
fn results_do_not_depend_on_what_was_computed_before() {
    init();
    let e = Ephemeris::new(Ayanamsa::Lahiri, NodeType::Mean);
    let solo = e.positions(JD).unwrap();

    // Exercise every other configuration, then come back.
    for aya in [Ayanamsa::Raman, Ayanamsa::Krishnamurti, Ayanamsa::TrueChitra,
                Ayanamsa::TrueRevati, Ayanamsa::Yukteshwar, Ayanamsa::LahiriIcrc] {
        for node in [NodeType::Mean, NodeType::True] {
            let other = Ephemeris::new(aya, node);
            let _ = other.positions(JD + 1000.0).unwrap();
            let _ = other.angles(JD, 60.0, 10.0, HouseSystem::Placidus);
            let _ = other.ayanamsa_value(JD);
        }
    }
    assert_eq!(e.positions(JD).unwrap(), solo, "prior work changed a later result");
}

#[test]
fn the_ephemeris_path_applies_to_threads_spawned_after_it_was_set() {
    // Regression for the Linux thread-local-state defect: Swiss Ephemeris
    // keeps its state per thread unless compiled with TLSOFF, so a fresh
    // thread would not see the path and strict mode would refuse to compute.
    init();
    let want = Ephemeris::new(Ayanamsa::Lahiri, NodeType::Mean).positions(JD).unwrap();
    let handles: Vec<_> = (0..8)
        .map(|_| thread::spawn(move || Ephemeris::new(Ayanamsa::Lahiri, NodeType::Mean).positions(JD)))
        .collect();
    for h in handles {
        let got = h.join().unwrap();
        assert_eq!(got.expect("a new thread could not use the ephemeris path"), want);
    }
}
