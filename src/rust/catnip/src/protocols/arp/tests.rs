// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use super::pdu::{
    ArpOp,
    ArpPdu,
};
use crate::{
    fail::Fail,
    protocols::ethernet,
    runtime::Runtime,
    test_helpers,
};
use futures::{
    task::{
        noop_waker_ref,
        Context,
    },
    FutureExt,
};
use hashbrown::HashMap;
use must_let::must_let;
use std::{
    future::Future,
    io::Cursor,
    task::Poll,
    time::{
        Duration,
        Instant,
    },
};

#[test]
fn immediate_reply() {
    // tests to ensure that an are request results in a reply.
    let now = Instant::now();
    let mut alice = test_helpers::new_alice(now);
    alice.import_arp_cache(HashMap::new());
    let mut bob = test_helpers::new_bob(now);
    bob.import_arp_cache(HashMap::new());
    let mut carrie = test_helpers::new_carrie(now);
    carrie.import_arp_cache(HashMap::new());

    // this test is written based on certain assumptions.
    let options = alice.rt().arp_options();
    assert_eq!(options.request_timeout, Duration::from_secs(1));

    let mut ctx = Context::from_waker(noop_waker_ref());
    let mut fut = alice.arp_query(test_helpers::CARRIE_IPV4).boxed_local();
    let now = now + Duration::from_micros(1);
    assert!(Future::poll(fut.as_mut(), &mut ctx).is_pending());

    alice.rt().advance_clock(now);
    let request = alice.rt().pop_frame();
    assert!(request.len() >= ethernet::MIN_PAYLOAD_SIZE);

    // bob hasn't heard of alice before, so he will ignore the request.
    info!("passing ARP request to bob (should be ignored)...");
    must_let!(let Err(Fail::Ignored { .. }) = bob.receive(&request));
    let cache = bob.export_arp_cache();
    assert!(cache.get(&test_helpers::ALICE_IPV4).is_none());

    carrie.receive(&request).unwrap();
    info!("passing ARP request to carrie...");
    let cache = carrie.export_arp_cache();
    assert_eq!(
        cache.get(&test_helpers::ALICE_IPV4),
        Some(&test_helpers::ALICE_MAC)
    );

    carrie.rt().advance_clock(now);
    let reply = carrie.rt().pop_frame();

    info!("passing ARP reply back to alice...");
    alice.receive(&reply).unwrap();
    let now = now + Duration::from_micros(1);
    alice.rt().advance_clock(now);
    must_let!(let Poll::Ready(Ok(link_addr)) = Future::poll(fut.as_mut(), &mut ctx));
    assert_eq!(test_helpers::CARRIE_MAC, link_addr);
}

#[test]
fn slow_reply() {
    // tests to ensure that an are request results in a reply.
    let mut now = Instant::now();
    let mut alice = test_helpers::new_alice(now);
    alice.import_arp_cache(HashMap::new());
    let mut bob = test_helpers::new_bob(now);
    bob.import_arp_cache(HashMap::new());
    let mut carrie = test_helpers::new_carrie(now);
    carrie.import_arp_cache(HashMap::new());

    // this test is written based on certain assumptions.
    let options = alice.rt().arp_options();
    assert!(options.retry_count > 0);
    assert_eq!(options.request_timeout, Duration::from_secs(1));

    let mut ctx = Context::from_waker(noop_waker_ref());
    let mut fut = alice.arp_query(test_helpers::CARRIE_IPV4).boxed_local();

    // move time forward enough to trigger a timeout.
    now += Duration::from_secs(1);
    alice.rt().advance_clock(now);
    assert!(Future::poll(fut.as_mut(), &mut ctx).is_pending());

    let request = alice.rt().pop_frame();
    assert!(request.len() >= ethernet::MIN_PAYLOAD_SIZE);

    // bob hasn't heard of alice before, so he will ignore the request.
    info!("passing ARP request to bob (should be ignored)...");
    must_let!(let Err(Fail::Ignored { .. }) = bob.receive(&request));

    let cache = bob.export_arp_cache();
    assert!(cache.get(&test_helpers::ALICE_IPV4).is_none());

    carrie.receive(&request).unwrap();
    info!("passing ARP request to carrie...");
    let cache = carrie.export_arp_cache();
    assert_eq!(
        cache.get(&test_helpers::ALICE_IPV4),
        Some(&test_helpers::ALICE_MAC)
    );

    carrie.rt().advance_clock(now);
    let reply = carrie.rt().pop_frame();

    info!("passing ARP reply back to alice...");
    alice.receive(&reply).unwrap();
    now += Duration::from_micros(1);
    alice.rt().advance_clock(now);
    must_let!(let Poll::Ready(Ok(link_addr)) = Future::poll(fut.as_mut(), &mut ctx));
    assert_eq!(test_helpers::CARRIE_MAC, link_addr);
}

#[test]
fn no_reply() {
    // tests to ensure that an are request results in a reply.
    let mut now = Instant::now();
    let alice = test_helpers::new_alice(now);
    alice.import_arp_cache(HashMap::new());
    let options = alice.rt().arp_options();

    assert_eq!(options.retry_count, 2);
    assert_eq!(options.request_timeout, Duration::from_secs(1));

    let mut ctx = Context::from_waker(noop_waker_ref());
    let mut fut = alice.arp_query(test_helpers::CARRIE_IPV4).boxed_local();
    assert!(Future::poll(fut.as_mut(), &mut ctx).is_pending());
    let bytes = alice.rt().pop_frame();
    let frame = ethernet::Frame::attach(bytes.as_slice()).unwrap();
    let arp = ArpPdu::read(&mut Cursor::new(frame.text())).unwrap();
    assert_eq!(arp.op, ArpOp::ArpRequest);

    for i in 0..options.retry_count {
        now += options.request_timeout;
        alice.rt().advance_clock(now);
        assert!(Future::poll(fut.as_mut(), &mut ctx).is_pending());
        info!("no_reply(): retry #{}", i + 1);
        let bytes = alice.rt().pop_frame();
        let frame = ethernet::Frame::attach(bytes.as_slice()).unwrap();
        let arp = ArpPdu::read(&mut Cursor::new(frame.text())).unwrap();
        assert_eq!(arp.op, ArpOp::ArpRequest);
    }

    // timeout
    now += options.request_timeout;
    alice.rt().advance_clock(now);

    must_let!(let Poll::Ready(Err(Fail::Timeout {})) = Future::poll(fut.as_mut(), &mut ctx));
}
