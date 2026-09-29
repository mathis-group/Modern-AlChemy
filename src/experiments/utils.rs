// Global Imports
use std::fmt::{Debug, Display};

// Package Imports
use crate::soupercollider::Soup;
use crate::record::Tape;
use crate::traits::{Particle, Collider, Generator};

impl<P, C, G> Soup<P, C, G> 
where
    P: Particle + Display + Clone,
    C: Collider<P> + Clone,
    G: Generator<P> + Clone
    {
    
    pub fn simulate_and_poll<F, R>(
        &mut self,
        n: usize,
        polling_interval: usize,
        log: bool,
        poller: F,
    ) -> Vec<R>
    where
        F: Fn(&Self) -> R,
    {
        let mut data: Vec<R> = Vec::new();
        for i in 0..n {
            let reaction = self.react(i, false);
            if (i % polling_interval) == 0 {
                data.push(poller(self))
            }
        }
        data
    }

    pub fn simulate_and_poll_with_killer<F, R>(
        &mut self,
        n: usize,
        polling_interval: usize,
        log: bool,
        killpoller: F,
    ) -> Vec<R>
    where
        F: Fn(&Self) -> (R, bool),
    {
        let mut data: Vec<R> = Vec::new();
        for i in 0..n {
            let _reaction = self.react(i, false);
            if (i % polling_interval) == 0 {
                let (datum, should_kill) = killpoller(self);
                data.push(datum);
                if should_kill {
                    return data;
                };
            }
        }
        data
    }

}