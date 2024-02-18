use crate::strategy::r#do::DoRule;
use crate::strategy::r#for::ForRule;
use crate::strategy::when::WhenRule;

struct RuleSet {
    for_rule: Box<dyn ForRule>,
    do_rule: Box<dyn DoRule>,
    when_rule: Box<dyn WhenRule>
}

// impl Default for RuleSet {
//     fn default() -> Self {
//         Self {
//             for_rule: Box::new(ForRule::default()),
//             do_rule: Box::new(DoRule::default()),
//             when_rule: Box::new(WhenRule::default())
//         }
//     }
//
// }

impl RuleSet {
    pub fn evaluate(&self) {

    }
}