use crate::Value;

/// Result row from a query
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Row {
    values: Vec<Value>,
}

impl Row {
    /// Create a new instance of [Row]
    #[must_use]
    pub fn new(values: Vec<Value>) -> Self {
        Self { values }
    }

    /// Get the first value of the row
    #[must_use]
    pub fn first(&self) -> Option<&Value> {
        self.values.first()
    }

    /// Get the value at the given index
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&Value> {
        self.values.get(index)
    }

    /// Check if the row is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Get the last value of the row
    #[must_use]
    pub fn last(&self) -> Option<&Value> {
        self.values.last()
    }

    /// Get the number of values in the row
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Get the values of the row
    #[must_use]
    pub fn values(&self) -> &Vec<Value> {
        &self.values
    }

    /// Get an iterator over the values of the row
    #[allow(dead_code)]
    fn iter(&self) -> std::slice::Iter<Value> {
        <&Self as IntoIterator>::into_iter(self)
    }
}

impl<'a> IntoIterator for &'a Row {
    type Item = &'a Value;
    type IntoIter = std::slice::Iter<'a, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.iter()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_row_new() {
        let values = vec![Value::String("foo".to_string())];
        let row = Row::new(values.clone());
        assert_eq!(row.values(), &values);
    }

    #[test]
    fn test_row_get() {
        let values = vec![Value::String("foo".to_string())];
        let row = Row::new(values.clone());
        let value = row.get(0).expect("no value");
        assert_eq!(value, &Value::String("foo".to_string()));
    }

    #[test]
    fn test_row_first() {
        let values = vec![
            Value::String("a".to_string()),
            Value::String("z".to_string()),
        ];
        let row = Row::new(values.clone());
        let value = row.first().expect("no value");
        assert_eq!(value, &Value::String("a".to_string()));
    }

    #[test]
    fn test_row_is_empty() {
        let values = vec![];
        let row = Row::new(values.clone());
        assert!(row.is_empty());
    }

    #[test]
    fn test_row_last() {
        let values = vec![
            Value::String("a".to_string()),
            Value::String("z".to_string()),
        ];
        let row = Row::new(values.clone());
        let value = row.last().expect("no value");
        assert_eq!(value, &Value::String("z".to_string()));
    }

    #[test]
    fn test_row_len() {
        let values = vec![Value::String("foo".to_string())];
        let row = Row::new(values.clone());
        assert_eq!(row.len(), 1);
    }

    #[test]
    fn test_row_iter() {
        let values = vec![Value::String("foo".to_string())];
        let row = Row::new(values.clone());
        let mut iterator = row.iter();
        assert_eq!(iterator.next(), Some(&Value::String("foo".to_string())));
        assert_eq!(iterator.next(), None);
    }
}
