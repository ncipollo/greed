pub trait BooleanWhen {
    fn when_false<F: FnOnce()>(self, f: F) -> Self;
    fn when_true<F: FnOnce()>(self, f: F) -> Self;
}

impl BooleanWhen for bool {
    fn when_false<F: FnOnce()>(self, f: F) -> Self {
        if !self {
            f();
        }
        return self
    }

    fn when_true<F: FnOnce()>(self, f: F) -> Self {
        if self {
            f();
        }
        return self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_when_false_false() {
        let mut value = 0;
        let result = false.when_false(|| value = 1);
        assert_eq!(result, false);
        assert_eq!(value, 1);
    }

    #[test]
    fn test_when_false_true() {
        let mut value = 0;
        let result = true.when_false(|| value = 1);
        assert_eq!(result, true);
        assert_eq!(value, 0);
    }

    #[test]
    fn test_when_true_false() {
        let mut value = 0;
        let result = false.when_true(|| value = 1);
        assert_eq!(result, false);
        assert_eq!(value, 0);
    }

    #[test]
    fn test_when_true_true() {
        let mut value = 0;
        let result = true.when_true(|| value = 1);
        assert_eq!(result, true);
        assert_eq!(value, 1);
    }
}