#[cfg(test)]
mod tests {
    use crate::{field_to_tag_value, get_field, SOH, TAG_DELIMETER};

    #[test]
    fn empty_field() {
        let result = get_field(&[], SOH);
        assert_eq!(result, []);
    }

    #[test]
    fn pipe_field() {
        let result = get_field(&[SOH], SOH);
        assert_eq!(result, []);
    }

    #[test]
    fn simple_field() {
        let result = get_field(&[12, 13, 15, SOH, 56, 78, SOH], SOH);
        assert_eq!(result, [12, 13, 15]);
    }

    #[test]
    fn no_separator() {
        let result = get_field(&[12, 13, 14, 15], SOH);
        assert_eq!(result, []);
    }

    #[test]
    #[should_panic]
    fn empty_field_to_tv() {
        let _res = field_to_tag_value(&[]);
    }

    #[test]
    #[should_panic]
    fn empty_value_to_tv() {
        let _res = field_to_tag_value(&[12, 13, 14, TAG_DELIMETER]);
    }

    #[test]
    #[should_panic]
    fn no_tv_separator() {
        let _res = field_to_tag_value(&[12, 13, 14]);
    }

    #[test]
    fn field_to_tv() {
        let res = field_to_tag_value(&[12, 13, 14, TAG_DELIMETER, 56, 67]);
        assert_eq!(res.0, [12, 13, 14]);
        assert_eq!(res.1, [56, 67]);
    }

    #[test]
    fn field_short_to_tv() {
        let res = field_to_tag_value(&[12, TAG_DELIMETER, 56]);
        assert_eq!(res.0, [12]);
        assert_eq!(res.1, [56]);
    }
}
