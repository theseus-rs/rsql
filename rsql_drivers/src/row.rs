use crate::Value;

/// Result row from a query
#[derive(Clone, Debug, Default)]
pub struct Row {
    values: Vec<Option<Value>>,
}

impl Row {
    /// Create a new instance of [Row]
    pub fn new(values: Vec<Option<Value>>) -> Self {
        Self { values }
    }

    /// Get the first value of the row
    pub fn first(&self) -> Option<&Value> {
        self.values.first().and_then(|v| v.as_ref())
    }

    /// Get the value at the given index
    pub fn get(&self, index: usize) -> Option<&Value> {
        self.values.get(index).and_then(|v| v.as_ref())
    }

    /// Check if the row is empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Get the last value of the row
    pub fn last(&self) -> Option<&Value> {
        self.values.last().and_then(|v| v.as_ref())
    }

    /// Get the number of values in the row
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Get the values of the row
    pub fn values(&self) -> &Vec<Option<Value>> {
        &self.values
    }
}

impl<'a> IntoIterator for &'a Row {
    type Item = &'a Option<Value>;
    type IntoIter = std::slice::Iter<'a, Option<Value>>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.iter()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_row_new() {
        let values = vec![Some(Value::String("foo".to_string()))];
        let row = Row::new(values.clone());
        assert_eq!(row.values(), &values);
    }

    #[test]
    fn test_row_get() {
        let values = vec![Some(Value::String("foo".to_string()))];
        let row = Row::new(values.clone());
        let value = row.get(0).expect("no value");
        assert_eq!(value, &Value::String("foo".to_string()));
    }

    #[test]
    fn test_row_first() {
        let values = vec![
            Some(Value::String("a".to_string())),
            Some(Value::String("z".to_string())),
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
            Some(Value::String("a".to_string())),
            Some(Value::String("z".to_string())),
        ];
        let row = Row::new(values.clone());
        let value = row.last().expect("no value");
        assert_eq!(value, &Value::String("z".to_string()));
    }

    #[test]
    fn test_row_len() {
        let values = vec![Some(Value::String("foo".to_string()))];
        let row = Row::new(values.clone());
        assert_eq!(row.len(), 1);
    }
}
