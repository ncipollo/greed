pub fn median(values: Vec<f64>) -> Option<f64> {
    let mut sorted = values.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let len = sorted.len();
    if len == 0 {
        return None;
    }
    if len % 2 == 0 {
        let mid = len / 2;
        Some((sorted[mid - 1] + sorted[mid]) / 2.0)
    } else {
        Some(sorted[len / 2])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_median_empty() {
        let values = vec![];
        assert_eq!(median(values), None);
    }

    #[test]
    fn test_median_even() {
        let values = vec![1.0, 2.0, 3.0, 4.0];
        assert_eq!(median(values), Some(2.5));
    }

    #[test]
    fn test_median_odd() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(median(values), Some(3.0));
    }
}
