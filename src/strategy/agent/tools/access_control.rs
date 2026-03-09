pub fn is_permitted(symbol: &str, allow: &[String], deny: &[String]) -> bool {
    let symbol_upper = symbol.to_uppercase();
    if !allow.is_empty() && !allow.iter().any(|s| s.to_uppercase() == symbol_upper) {
        return false;
    }
    if !deny.is_empty() && deny.iter().any(|s| s.to_uppercase() == symbol_upper) {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_lists_permits_all() {
        assert!(is_permitted("VTI", &[], &[]));
    }

    #[test]
    fn allow_list_permits_listed_symbol() {
        assert!(is_permitted("VTI", &["VTI".to_string()], &[]));
    }

    #[test]
    fn allow_list_blocks_unlisted_symbol() {
        assert!(!is_permitted("VXUS", &["VTI".to_string()], &[]));
    }

    #[test]
    fn allow_list_case_insensitive() {
        assert!(is_permitted("vti", &["VTI".to_string()], &[]));
    }

    #[test]
    fn deny_list_blocks_symbol() {
        assert!(!is_permitted("VTI", &[], &["VTI".to_string()]));
    }

    #[test]
    fn deny_list_case_insensitive() {
        assert!(!is_permitted("VTI", &[], &["vti".to_string()]));
    }

    #[test]
    fn deny_list_permits_other_symbols() {
        assert!(is_permitted("VXUS", &[], &["VTI".to_string()]));
    }
}
