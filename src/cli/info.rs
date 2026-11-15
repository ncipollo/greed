use clap::{Args, ValueEnum};

#[derive(Args, Debug)]
pub struct InfoArgs {
    /// Topic to show documentation for. Omit to list all topics.
    #[arg(value_name = "TOPIC", value_enum)]
    pub topic: Option<InfoTopic>,
}

/// Valid `greed info` topics. Variants must stay in sync with
/// `greed::info::TOPICS` (enforced by a test below).
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum InfoTopic {
    /// What greed is, its CLI commands, and run vs server
    General,
    /// Top-level greed.toml config
    Config,
    /// Simple CSV config format
    ConfigSimple,
    /// Strategy entries
    Strategy,
    /// Tactic buy/sell rules
    Tactic,
    /// AI agent config
    Agent,
    /// HTTP server endpoints
    Server,
}

impl InfoTopic {
    /// The registry name for this topic, as shown in `greed info`.
    pub fn name(&self) -> String {
        self.to_possible_value()
            .expect("no variant is skipped")
            .get_name()
            .to_string()
    }
}

/// Prints the directory when no topic is given, otherwise the topic page.
pub fn render(topic: Option<InfoTopic>) -> String {
    match topic {
        None => greed::info::directory(),
        Some(topic) => greed::info::topic(&topic.name()).expect("every InfoTopic has a page"),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cli::{Cli, Command};
    use clap::Parser;

    #[test]
    fn enum_variants_match_registry() {
        let enum_names: Vec<String> = InfoTopic::value_variants()
            .iter()
            .map(|t| t.name())
            .collect();
        assert_eq!(enum_names, greed::info::names());
    }

    #[test]
    fn parses_without_topic() {
        let cli = Cli::try_parse_from(["greed", "info"]).expect("should parse");
        let Command::Info(args) = cli.command else {
            panic!("expected Info command")
        };
        assert_eq!(args.topic, None);
    }

    #[test]
    fn parses_topic() {
        let cli = Cli::try_parse_from(["greed", "info", "config-simple"]).expect("should parse");
        let Command::Info(args) = cli.command else {
            panic!("expected Info command")
        };
        assert_eq!(args.topic, Some(InfoTopic::ConfigSimple));
    }

    #[test]
    fn rejects_unknown_topic() {
        assert!(Cli::try_parse_from(["greed", "info", "bogus"]).is_err());
    }

    #[test]
    fn render_without_topic_is_directory() {
        assert_eq!(render(None), greed::info::directory());
    }

    #[test]
    fn render_every_topic() {
        for topic in InfoTopic::value_variants() {
            assert!(!render(Some(*topic)).is_empty());
        }
    }
}
