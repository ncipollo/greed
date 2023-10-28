pub trait ConfigRuleMerge: Sized {
    fn merge(&self, higher: Option<Self>) -> Self;
}