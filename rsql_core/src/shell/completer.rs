use crate::shell::helper::ReplHelper;
use rustyline::completion::{Candidate, Completer, Pair};
use rustyline::Context;
use std::sync::LazyLock;

static CANDIDATES: LazyLock<Vec<Pair>> = LazyLock::new(init_candidates);

#[allow(clippy::too_many_lines)]
fn init_candidates() -> Vec<Pair> {
    vec![
        Pair {
            display: "SELECT - Retrieve data from a table".to_string(),
            replacement: "SELECT".to_string(),
        },
        Pair {
            display: "FROM - Specify the source table".to_string(),
            replacement: "FROM".to_string(),
        },
        Pair {
            display: "WHERE - Filter the result set".to_string(),
            replacement: "WHERE".to_string(),
        },
        Pair {
            display: "ORDER BY - Sort the result set".to_string(),
            replacement: "ORDER BY".to_string(),
        },
        Pair {
            display: "GROUP BY - Group the result set".to_string(),
            replacement: "GROUP BY".to_string(),
        },
        Pair {
            display: "HAVING - Filter groups".to_string(),
            replacement: "HAVING".to_string(),
        },
        Pair {
            display: "LIMIT - Limit the result set".to_string(),
            replacement: "LIMIT".to_string(),
        },
        Pair {
            display: "OFFSET - Skip rows before starting".to_string(),
            replacement: "OFFSET".to_string(),
        },
        Pair {
            display: "UNION - Combine result sets".to_string(),
            replacement: "UNION".to_string(),
        },
        Pair {
            display: "INTERSECT - Return common rows".to_string(),
            replacement: "INTERSECT".to_string(),
        },
        Pair {
            display: "EXCEPT - Return rows that are not in the second result set".to_string(),
            replacement: "EXCEPT".to_string(),
        },
        Pair {
            display: "JOIN - Combine rows from two or more tables".to_string(),
            replacement: "JOIN".to_string(),
        },
        Pair {
            display: "INNER JOIN - Return rows when there is a match in both tables".to_string(),
            replacement: "INNER JOIN".to_string(),
        },
        Pair {
            display: "LEFT JOIN - Return all rows from the left table, and the matched rows from the right table".to_string(),
            replacement: "LEFT JOIN".to_string(),
        },
        Pair {
            display: "RIGHT JOIN - Return all rows from the right table, and the matched rows from the left table".to_string(),
            replacement: "RIGHT JOIN".to_string(),
        },
        Pair {
            display: "FULL JOIN - Return all rows when there is a match in one of the tables".to_string(),
            replacement: "FULL JOIN".to_string(),
        },
        Pair {
            display: "CROSS JOIN - Return the Cartesian product of the sets of rows from the joined tables".to_string(),
            replacement: "CROSS JOIN".to_string(),
        },
        Pair {
            display: "NATURAL JOIN - Return all rows from the joined tables that have a match in both tables".to_string(),
            replacement: "NATURAL JOIN".to_string(),
        },
        Pair {
            display: "ON - Specify the join condition".to_string(),
            replacement: "ON".to_string(),
        },
        Pair {
            display: "USING - Specify the columns for joining".to_string(),
            replacement: "USING".to_string(),
        },
        Pair {
            display: "SET - Modify data in a table".to_string(),
            replacement: "SET".to_string(),
        },
        Pair {
            display: "INSERT INTO - Insert new rows into a table".to_string(),
            replacement: "INSERT INTO".to_string(),
        },
        Pair {
            display: "VALUES - Specify the values to insert".to_string(),
            replacement: "VALUES".to_string(),
        },
        Pair {
            display: "UPDATE - Modify existing rows in a table".to_string(),
            replacement: "UPDATE".to_string(),
        },
        Pair {
            display: "DELETE FROM - Delete rows from a table".to_string(),
            replacement: "DELETE FROM".to_string(),
        },
        Pair {
            display: "CREATE TABLE - Create a new table".to_string(),
            replacement: "CREATE TABLE".to_string(),
        },
        Pair {
            display: "ALTER TABLE - Modify a table".to_string(),
            replacement: "ALTER TABLE".to_string(),
        },
        Pair {
            display: "DROP TABLE - Delete a table".to_string(),
            replacement: "DROP TABLE".to_string(),
        },
        Pair {
            display: "CREATE INDEX - Create a new index".to_string(),
            replacement: "CREATE INDEX".to_string(),
        },
        Pair {
            display: "DROP INDEX - Delete an index".to_string(),
            replacement: "DROP INDEX".to_string(),
        },
        Pair {
            display: "CREATE VIEW - Create a new view".to_string(),
            replacement: "CREATE VIEW".to_string(),
        },
        Pair {
            display: "DROP VIEW - Delete a view".to_string(),
            replacement: "DROP VIEW".to_string(),
        },
        Pair {
            display: "CREATE SCHEMA - Create a new schema".to_string(),
            replacement: "CREATE SCHEMA".to_string(),
        },
        Pair {
            display: "DROP SCHEMA - Delete a schema".to_string(),
            replacement: "DROP SCHEMA".to_string(),
        },
        Pair {
            display: "GRANT - Give privileges to a user".to_string(),
            replacement: "GRANT".to_string(),
        },
        Pair {
            display: "REVOKE - Take privileges away from a user".to_string(),
            replacement: "REVOKE".to_string(),
        },
        Pair {
            display: "COMMIT - Save the changes".to_string(),
            replacement: "COMMIT".to_string(),
        },
        Pair {
            display: "ROLLBACK - Rollback the changes".to_string(),
            replacement: "ROLLBACK".to_string(),
        },
    ]
}

impl Completer for ReplHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context,
    ) -> Result<(usize, Vec<Pair>), rustyline::error::ReadlineError> {
        let start = line[..pos]
            .rfind(|c: char| c.is_whitespace())
            .map_or(0, |p| p + 1);
        let word = &line[start..pos];
        let candidates = CANDIDATES
            .iter()
            .filter(|c| c.replacement().starts_with(word))
            .cloned()
            .collect();
        Ok((start, candidates))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::configuration::Configuration;
    use rustyline::history::DefaultHistory;

    #[test]
    fn test_complete() -> anyhow::Result<()> {
        let configuration = Configuration::default();
        let helper = ReplHelper::new(&configuration);
        let history = DefaultHistory::new();
        let context = Context::new(&history);
        let (start, candidates) = helper.complete("SEL", 3, &context)?;
        assert_eq!(start, 0);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].replacement(), "SELECT");
        Ok(())
    }
}
