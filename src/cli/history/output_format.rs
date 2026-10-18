use clap::Args;
use greed::output_format::OutputFormat;

#[derive(Args, Debug)]
pub struct OutputFormatArgs {
    /// Controls how output is printed: "text" (default, human-readable) or "json" (machine-readable).
    #[arg(long, default_value = "text")]
    pub format: OutputFormat,
}
