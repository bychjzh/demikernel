// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use crate::{
    protocols::{arp, ethernet2::MacAddress, tcp},
    rand::Seed,
};
use rand::{thread_rng, Rng};
use std::net::Ipv4Addr;

#[derive(Clone, Debug)]
pub struct Options {
    pub arp: arp::Options,
    pub my_ipv4_addr: Ipv4Addr,
    pub my_link_addr: MacAddress,
    pub rng_seed: Seed,
    pub tcp: tcp::Options,
}

impl Default for Options {
    fn default() -> Self {
        let mut rng_seed = Seed::default();
        thread_rng().fill(rng_seed.as_mut());
        Options {
            arp: arp::Options::default(),
            my_ipv4_addr: Ipv4Addr::new(0, 0, 0, 0),
            my_link_addr: MacAddress::nil(),
            rng_seed,
            tcp: tcp::Options::default(),
        }
    }
}

impl Options {
    pub fn arp(mut self, value: arp::Options) -> Self {
        self.arp = value;
        self
    }

    pub fn my_ipv4_addr(mut self, value: Ipv4Addr) -> Self {
        assert!(!value.is_unspecified());
        assert!(!value.is_broadcast());
        self.my_ipv4_addr = value;
        self
    }

    pub fn my_link_addr(mut self, value: MacAddress) -> Self {
        assert!(!value.is_nil());
        assert!(!value.is_broadcast());
        self.my_link_addr = value;
        self
    }

    pub fn rng_seed(mut self, value: Seed) -> Self {
        self.rng_seed = value;
        self
    }

    pub fn tcp(mut self, value: tcp::Options) -> Self {
        self.tcp = value;
        self
    }

    pub fn randomize_rng_seed(&mut self) {
        thread_rng().fill(self.rng_seed.as_mut());
    }
}
