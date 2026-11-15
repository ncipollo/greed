//! Agent-oriented documentation served by `greed info [TOPIC]`.
//!
//! A single table ([`TOPICS`]) drives both the bare-`greed info` directory and
//! page rendering. When changing config formats or HTTP endpoints, update the
//! relevant topic page; tests parse the embedded examples against the real
//! config types and check every `ServerRoute` appears in the server page.

mod agent;
mod config;
mod config_simple;
mod general;
mod server;
mod strategy;
mod tactic;

pub struct Topic {
    pub name: &'static str,
    pub summary: &'static str,
    render: fn() -> String,
}

pub const TOPICS: &[Topic] = &[
    Topic {
        name: "general",
        summary: "What greed is, its CLI commands, and run vs server",
        render: general::render,
    },
    Topic {
        name: "config",
        summary: "Top-level greed.toml config (platform, interval, strategies, tactics)",
        render: config::render,
    },
    Topic {
        name: "config-simple",
        summary: "Simple CSV config format (one row per asset)",
        render: config_simple::render,
    },
    Topic {
        name: "strategy",
        summary: "Strategy entries: local tactic files and agent strategies",
        render: strategy::render,
    },
    Topic {
        name: "tactic",
        summary: "Tactic buy/sell rules: for, when, do",
        render: tactic::render,
    },
    Topic {
        name: "agent",
        summary: "AI agent config: prompt, provider, allow/deny, tools",
        render: agent::render,
    },
    Topic {
        name: "server",
        summary: "HTTP server endpoints, parameters, and responses",
        render: server::render,
    },
];

/// The directory page listing every topic with its one-line summary.
pub fn directory() -> String {
    let width = TOPICS.iter().map(|t| t.name.len()).max().unwrap_or(0);
    let mut out = String::from("TOPICS\n");
    for topic in TOPICS {
        out.push_str(&format!(
            "  {:<width$}  {}\n",
            topic.name,
            topic.summary,
            width = width
        ));
    }
    out.push_str("\nrun: greed info <topic>\n");
    out
}

/// Renders the page for `name`, or `None` if there is no such topic.
pub fn topic(name: &str) -> Option<String> {
    TOPICS.iter().find(|t| t.name == name).map(|t| (t.render)())
}

pub fn names() -> Vec<&'static str> {
    TOPICS.iter().map(|t| t.name).collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn names_are_unique_and_lowercase() {
        let names = names();
        let unique: HashSet<_> = names.iter().collect();
        assert_eq!(unique.len(), names.len());
        for name in names {
            assert_eq!(name, name.to_lowercase());
            assert!(!name.contains(' '));
        }
    }

    #[test]
    fn every_topic_renders_non_empty_page() {
        for topic in TOPICS {
            assert!(!(topic.render)().trim().is_empty(), "{}", topic.name);
            assert!(!topic.summary.is_empty(), "{}", topic.name);
        }
    }

    #[test]
    fn directory_lists_every_topic_and_ends_with_hint() {
        let directory = directory();
        for topic in TOPICS {
            assert!(directory.contains(topic.name));
            assert!(directory.contains(topic.summary));
        }
        assert!(directory.trim_end().ends_with("run: greed info <topic>"));
    }

    #[test]
    fn topic_returns_page_for_known_name() {
        assert!(topic("server").is_some());
    }

    #[test]
    fn topic_returns_none_for_unknown_name() {
        assert!(topic("nope").is_none());
    }
}
