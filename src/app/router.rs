use std::{io, sync::Arc};

use anyhow::{
    Result,
    anyhow
};
use ipnet::{IpAddrRange, IpNet};
use log::{warn, debug};

use crate::{proxy::{Session, Address}, config::Rule};

// https://v2ray.com/chapter_02/03_routing.html

pub trait ConditionMatcher: Sync + Send + Unpin {
    fn apply(&self, sess: &Session) -> bool;
}

struct MatcherRule {
    target: String,
    matcher: Box<dyn ConditionMatcher>
}

impl MatcherRule {
    pub fn new(target: String, matcher: Box<dyn ConditionMatcher>) -> MatcherRule {
        MatcherRule {
            target,
            matcher
        }
    }
}


pub struct Router {
    rules: Vec<MatcherRule>
}

macro_rules! try_rule {
    ($matcher: expr) => {
        match $matcher {
            Ok(x) => x,
            Err(err) => {
                warn!("{}", err);
                continue;
            }
        }
    };
}

impl Router {
    pub fn new(rules: Vec<Rule>) -> Router {
        let mut router = Self {
            rules: Vec::new()
        };
        for rule in rules.iter() {
            if let Some(ref name) = rule.domain {
                let matcher = try_rule!(DomainMatcher::new(name.clone()));
                router.rules.push(MatcherRule::new(rule.target.clone(), Box::new(matcher)))
            }
            if let Some(ref cidr) = rule.ip {
                let matcher = try_rule!(IpCidrMatcher::new(cidr.clone()));
                router.rules.push(MatcherRule::new(rule.target.clone(), Box::new(matcher)));
            }
        }
        return router;
    }

    pub fn route(&self, sess: &Session) -> Option<String> {
        for rule in &self.rules {
            if rule.matcher.apply(&sess) {
                return Some(rule.target.clone())
            }
        }
        debug!("no routing found {:?}", sess);
        return None
    }
}

pub struct DomainMatcher {
    value: Vec<String>
}

impl DomainMatcher {
    pub fn new(value: Vec<String>)-> io::Result<DomainMatcher> {
        Ok(Self {
            value: value,
        })
    }
}

impl ConditionMatcher for DomainMatcher {
    fn apply(&self, sess: &Session) -> bool {
        return match &sess.destination {
            Address::Domain(name, _) => {
                self.value.contains(name)
            },
            _ => false
        }
    }
}

pub struct IpCidrMatcher {
    value: Vec<IpNet>
}

impl IpCidrMatcher {
    pub fn new(value: Vec<String>) -> Result<IpCidrMatcher> {
        let mut ips = Vec::new();
        for ip in value.iter() {
            let cidr = match ip.parse::<IpNet>() {
                Ok(x) => x,
                Err(err) => {
                    return Err(anyhow!("invalid cidr {} {}", ip, err))
                }
            };
            ips.push(cidr);
        }
        
        Ok(Self {
            value: ips
        })
    }
}

impl ConditionMatcher for IpCidrMatcher {
    fn apply(&self, sess: &Session) -> bool {
        match sess.destination {
            Address::Ip(ip) => {
                let i = &IpNet::from(ip.ip());
                self.value.contains(i)
            },
            _ => false
        }
    }
}