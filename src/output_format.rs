use clap::ValueEnum;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
}

#[cfg(test)]
mod test {
    use crate::output_format::OutputFormat;

    #[test]
    fn default() {
        let default = OutputFormat::default();
        assert_eq!(OutputFormat::Text, default)
    }
}
