pub trait QuoteFetcherConfig {
    fn should_fetch_quotes(&self) -> bool;
}
