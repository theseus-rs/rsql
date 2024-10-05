use rsql_drivers::{Metadata, Table};
use rustyline::completion::{Candidate, Completer, Pair};
use rustyline::Context;
use sqlparser::keywords::Keyword;
use sqlparser::tokenizer::{Token, TokenWithLocation, Tokenizer};
use std::matches;
use std::sync::LazyLock;
use tracing::debug;

use crate::configuration::Configuration;

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
            display: "INTO - Insert new rows into a table".to_string(),
            replacement: "INTO".to_string(),
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

#[derive(Debug)]
enum Suggestion {
    Keyword(String),
    Schema,
    Table(String),
    TableInQuery(String),
    TableColumn(String),
}

impl Default for Suggestion {
    fn default() -> Self {
        Self::Keyword(String::new())
    }
}

#[derive(Debug)]
pub struct ReplCompleter {
    metadata: Metadata,
    smart_completions: bool,
}

impl ReplCompleter {
    pub fn with_config(configuration: &Configuration, metadata: Metadata) -> Self {
        let mut completer = Self::new(metadata);
        completer.smart_completions = configuration.smart_completions;
        completer
    }

    pub fn new(metadata: Metadata) -> Self {
        Self {
            metadata,
            smart_completions: true,
        }
    }

    /// identifies table names and aliases in `tokens`
    /// returns vector of tuples with `Table` and optional alias String
    fn tables_in_query(&self, tokens: &[TokenWithLocation]) -> Vec<(&Table, Option<String>)> {
        let tokens_no_location: Vec<_> = tokens
            .iter()
            .filter_map(|token| {
                if matches!(token.token, Token::Whitespace(_)) {
                    None
                } else {
                    Some(&token.token)
                }
            })
            .collect();
        let table_aliases: Vec<_> = tokens_no_location
            .windows(3)
            .filter_map(|window| match (window[0], window[1], window[2]) {
                (Token::Word(table), Token::Word(as_keyword), Token::Word(alias))
                    if as_keyword.keyword == Keyword::AS =>
                {
                    Some((table.value.clone(), alias.value.clone()))
                }
                (Token::Word(table), Token::Word(alias), _)
                    if alias.keyword == Keyword::NoKeyword
                        && table.keyword == Keyword::NoKeyword =>
                {
                    Some((table.value.clone(), alias.value.clone()))
                }
                _ => None,
            })
            .collect();

        debug!("Table aliases found: {:?}", table_aliases);

        let tables = if let Some(schema) = self.metadata.current_schema() {
            tokens_no_location
                .iter()
                .filter_map(|token| {
                    if let Token::Word(word) = token {
                        schema.get(&word.value)
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            vec![]
        };

        tables
            .into_iter()
            .map(|table| {
                if let Some((_, alias)) = table_aliases
                    .iter()
                    .find(|(table_name, _)| table.name() == table_name)
                {
                    (table, Some(alias.clone()))
                } else {
                    (table, None)
                }
            })
            .collect()
    }

    fn suggest_type(
        tokens: &[TokenWithLocation],
        token_at_cursor: &TokenWithLocation,
        tables: &[(&Table, Option<String>)],
    ) -> Suggestion {
        let token_string = token_at_cursor.token.to_string().trim().to_string();
        debug!("token at cursor: {token_at_cursor}");
        let Some(token_idx) = tokens
            .iter()
            .enumerate()
            .rfind(|(_, token)| token.location == token_at_cursor.location)
            .map(|(i, _)| i.saturating_sub(1))
        else {
            return Suggestion::default();
        };

        let token_before_cursor_search = tokens
            .iter()
            .enumerate()
            .rfind(|(i, token)| *i <= token_idx && !matches!(token.token, Token::Whitespace(_)));

        let Some((token_before_cursor_idx, token_before_cursor)) = token_before_cursor_search
        else {
            return Suggestion::Keyword(token_string);
        };

        Self::suggest_on_last_token(
            tokens,
            tables,
            token_at_cursor,
            token_before_cursor,
            token_before_cursor_idx,
        )
    }

    fn suggest_on_last_token(
        tokens: &[TokenWithLocation],
        tables: &[(&Table, Option<String>)],
        token_at_cursor: &TokenWithLocation,
        previous_token: &TokenWithLocation,
        previous_token_idx: usize,
    ) -> Suggestion {
        let token_string = token_at_cursor.token.to_string();
        debug!("token before cursor: {previous_token}");
        match &previous_token.token {
            Token::Comma | Token::Plus | Token::Eq | Token::Mul | Token::Div => {
                if let Some((previous_idx, previous_keyword)) =
                    find_previous_keyword(tokens, previous_token_idx)
                {
                    Self::suggest_on_last_token(
                        tokens,
                        tables,
                        token_at_cursor,
                        previous_keyword,
                        previous_idx,
                    )
                } else {
                    Suggestion::Keyword(token_string)
                }
            }
            Token::Word(word) => match word.keyword {
                Keyword::BY => {
                    if let Some((previous_idx, previous_keyword)) =
                        find_previous_keyword(tokens, previous_token_idx)
                    {
                        Self::suggest_on_last_token(
                            tokens,
                            tables,
                            token_at_cursor,
                            previous_keyword,
                            previous_idx,
                        )
                    } else {
                        Suggestion::Keyword(token_string)
                    }
                }
                Keyword::INSERT => Suggestion::Keyword("INTO".into()),
                Keyword::DELETE => Suggestion::Keyword("FROM".into()),
                Keyword::ON => Suggestion::TableInQuery(token_string.trim().to_string()),
                Keyword::FROM
                | Keyword::JOIN
                | Keyword::INTO
                | Keyword::UPDATE
                | Keyword::TABLE
                | Keyword::TRUNCATE => Suggestion::Table(token_string.trim().to_string()),
                Keyword::DATABASE | Keyword::USE => Suggestion::Schema,
                Keyword::NoKeyword
                | Keyword::SELECT
                | Keyword::WHERE
                | Keyword::ORDER
                | Keyword::GROUP => {
                    if tables.len() == 1 {
                        Suggestion::TableColumn(tables[0].0.name().to_string())
                    } else if let Some((table, _)) = tables.iter().find(|(t, a)| {
                        t.name() == word.value || a.as_ref().is_some_and(|a| a == &word.value)
                    }) {
                        Suggestion::TableColumn(table.name().to_string())
                    } else {
                        Suggestion::Keyword(token_string)
                    }
                }
                _ => Suggestion::Keyword(token_string),
            },
            _ => Suggestion::Keyword(token_string),
        }
    }

    fn matches_for_token(
        &self,
        token_at_cursor: &TokenWithLocation,
        tables: &[(&Table, Option<String>)],
        suggestion: Suggestion,
    ) -> Vec<Pair> {
        debug!("Suggestion: {suggestion:?}");
        match suggestion {
            Suggestion::Keyword(starts_with) => CANDIDATES
                .iter()
                .filter(|c| c.replacement().starts_with(&starts_with))
                .cloned()
                .collect(),
            Suggestion::Table(starts_with) => {
                if let Some(schema) = self.metadata.current_schema() {
                    schema
                        .tables()
                        .iter()
                        .filter(|table| table.name().starts_with(starts_with.trim()))
                        .map(|table| Pair {
                            display: format!("Table: {}", table.name()),
                            replacement: table.name().to_string(),
                        })
                        .collect()
                } else {
                    vec![]
                }
            }
            Suggestion::TableColumn(table_name) => tables
                .iter()
                .find(|(table, _)| table.name() == table_name)
                .map(|(table, _)| {
                    let prefix = match &token_at_cursor.token {
                        Token::Period => String::new(),
                        token => token.to_string().trim().to_string(),
                    };
                    table
                        .columns()
                        .into_iter()
                        .filter(|column| column.name().starts_with(&prefix))
                        .map(|column| Pair {
                            display: format!("Column: {}", column.name()),
                            replacement: column.name().to_string(),
                        })
                        .collect::<Vec<Pair>>()
                })
                .unwrap_or_default(),
            Suggestion::TableInQuery(prefix) => tables
                .iter()
                .flat_map(|(table, alias)| {
                    if let Some(alias) = alias {
                        vec![alias.to_string(), table.name().to_string()]
                    } else {
                        vec![table.name().to_string()]
                    }
                })
                .filter(|table_alias| table_alias.starts_with(&prefix))
                .map(|table_alias| Pair {
                    display: format!("Table: {table_alias}"),
                    replacement: table_alias,
                })
                .collect(),
            Suggestion::Schema => self
                .metadata
                .schemas()
                .iter()
                .map(|schema| Pair {
                    display: format!("Schema: {}", schema.name()),
                    replacement: schema.name().to_string(),
                })
                .collect(),
        }
    }

    fn get_completions(
        &self,
        token_at_cursor: &TokenWithLocation,
        tokens: &[TokenWithLocation],
    ) -> Vec<Pair> {
        let tables = self.tables_in_query(tokens);
        let suggestion = Self::suggest_type(tokens, token_at_cursor, &tables);
        self.matches_for_token(token_at_cursor, &tables, suggestion)
    }
}

impl Completer for ReplCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context,
    ) -> Result<(usize, Vec<Self::Candidate>), rustyline::error::ReadlineError> {
        if !self.smart_completions {
            let start = line[..pos]
                .rfind(|c: char| c.is_whitespace())
                .map_or(0, |p| p + 1);
            let word = &line[start..pos];
            let candidates = CANDIDATES
                .iter()
                .filter(|c| c.replacement().starts_with(word))
                .cloned()
                .collect();
            return Ok((start, candidates));
        }

        let cursor_location = u64::try_from(pos).unwrap_or(0);
        debug!("looking for completions line: {line}, pos: {cursor_location}");
        let dialect = self.metadata.dialect();
        let tokens: Vec<_> = Tokenizer::new(dialect.as_ref(), line)
            .tokenize_with_location()
            .unwrap_or_default();

        let Some(token_at_cursor) = tokens
            .iter()
            .enumerate()
            .rfind(|(_, token)| token.location.column <= cursor_location)
            .map(|token| token.1.to_owned())
        else {
            return Ok((0, CANDIDATES.clone()));
        };

        let start = usize::try_from(token_at_cursor.location.column.saturating_sub(
            match token_at_cursor.token {
                Token::Period | Token::Whitespace(_) => 0,
                _ => 1,
            },
        ))
        .unwrap_or(pos);
        let candidates = self.get_completions(&token_at_cursor, &tokens);

        Ok((start, candidates))
    }
}

fn find_previous_keyword(
    tokens: &[TokenWithLocation],
    index: usize,
) -> Option<(usize, &TokenWithLocation)> {
    tokens.iter().enumerate().rfind(|(i, token)| {
        *i < index
            && match &token.token {
                Token::Word(word) => !matches!(word.keyword, Keyword::NoKeyword | Keyword::ID),
                _ => false,
            }
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::configuration::Configuration;
    use crate::shell::helper::ReplHelper;
    use rsql_drivers::{Column, Schema};
    use rustyline::history::DefaultHistory;
    use sqlparser::{dialect::GenericDialect, tokenizer::Word};

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

    #[test]
    fn test_keyword_completion() {
        let metadata = create_mock_metadata();
        let completer = ReplCompleter::new(metadata);
        let (start, candidates) = completer
            .complete("SEL", 3, &Context::new(&DefaultHistory::new()))
            .expect("valid completion");

        assert_eq!(start, 0);
        assert!(candidates.iter().any(|c| c.replacement() == "SELECT"));
    }

    #[test]
    fn test_table_completion() {
        let metadata = create_mock_metadata();
        let completer = ReplCompleter::new(metadata);
        let (start, candidates) = completer
            .complete("SELECT * FROM u", 15, &Context::new(&DefaultHistory::new()))
            .expect("valid completion");

        assert_eq!(start, 14);
        assert!(candidates.iter().any(|c| c.replacement() == "users"));
    }

    #[test]
    fn test_column_completion() {
        let metadata = create_mock_metadata();
        let completer = ReplCompleter::new(metadata);
        let (start, candidates) = completer
            .complete(
                "SELECT id, n FROM users",
                12,
                &Context::new(&DefaultHistory::new()),
            )
            .expect("valid complete request failed");

        assert_eq!(start, 11);
        assert!(candidates.iter().any(|c| c.replacement() == "name"));
    }

    #[test]
    fn test_table_column_completion() {
        let metadata = create_mock_metadata();
        let completer = ReplCompleter::new(metadata);
        let (start, candidates) = completer
            .complete("SELECT users.", 13, &Context::new(&DefaultHistory::new()))
            .expect("valid completion");

        assert_eq!(start, 13);
        assert!(candidates.iter().any(|c| c.replacement() == "id"));
        assert!(candidates.iter().any(|c| c.replacement() == "name"));
        assert!(candidates.iter().any(|c| c.replacement() == "email"));
    }

    #[test]
    fn test_completion_with_alias() {
        let metadata = create_mock_metadata();
        let completer = ReplCompleter::new(metadata);
        let (start, candidates) = completer
            .complete(
                "SELECT u. FROM users u",
                9,
                &Context::new(&DefaultHistory::new()),
            )
            .expect("complete should not fail in this ");

        assert_eq!(start, 9);
        assert!(candidates.iter().any(|c| c.replacement() == "id"));
        assert!(candidates.iter().any(|c| c.replacement() == "name"));
        assert!(candidates.iter().any(|c| c.replacement() == "email"));
    }

    #[test]
    fn test_schema_completion() {
        let metadata = create_mock_metadata();
        let completer = ReplCompleter::new(metadata);
        let (start, candidates) = completer
            .complete("USE ", 4, &Context::new(&DefaultHistory::new()))
            .expect("valid completion call");

        assert_eq!(start, 4);
        assert!(candidates.iter().any(|c| c.replacement() == "public"));
    }

    #[test]
    fn test_completion_after_where() {
        let metadata = create_mock_metadata();
        let completer = ReplCompleter::new(metadata);
        let (start, candidates) = completer
            .complete(
                "SELECT * FROM users WHERE ",
                26,
                &Context::new(&DefaultHistory::new()),
            )
            .expect("valid completion call");

        assert_eq!(start, 26);
        assert!(candidates.iter().any(|c| c.replacement() == "id"));
        assert!(candidates.iter().any(|c| c.replacement() == "name"));
        assert!(candidates.iter().any(|c| c.replacement() == "email"));
    }

    #[test]
    fn test_completion_after_join() {
        let metadata = create_mock_metadata();
        let completer = ReplCompleter::new(metadata);
        let (start, candidates) = completer
            .complete(
                "SELECT * FROM users JOIN ",
                25,
                &Context::new(&DefaultHistory::new()),
            )
            .expect("valid completion call");

        assert_eq!(start, 25);
        assert!(candidates.iter().any(|c| c.replacement() == "orders"));
    }

    #[test]
    fn test_completion_after_on() {
        let metadata = create_mock_metadata();
        let completer = ReplCompleter::new(metadata);
        let (start, candidates) = completer
            .complete(
                "SELECT * FROM users JOIN orders ON ",
                35,
                &Context::new(&DefaultHistory::new()),
            )
            .expect("valid completion call");

        assert_eq!(start, 35);
        assert!(candidates.iter().any(|c| c.replacement() == "users"));
        assert!(candidates.iter().any(|c| c.replacement() == "orders"));
    }

    #[test]
    fn test_completion_after_on_with_aliases() {
        let metadata = create_mock_metadata();
        let completer = ReplCompleter::new(metadata);
        let (start, candidates) = completer
            .complete(
                "SELECT * FROM users u JOIN orders o ON ",
                39,
                &Context::new(&DefaultHistory::new()),
            )
            .expect("valid completion call");

        assert_eq!(start, 39);
        assert!(candidates.iter().any(|c| c.replacement() == "u"));
        assert!(candidates.iter().any(|c| c.replacement() == "o"));
        assert!(candidates.iter().any(|c| c.replacement() == "users"));
        assert!(candidates.iter().any(|c| c.replacement() == "orders"));

        let (start, candidates) = completer
            .complete(
                "SELECT * FROM users AS u JOIN orders AS o ON ",
                45,
                &Context::new(&DefaultHistory::new()),
            )
            .expect("valid completion call");
        assert_eq!(start, 45);
        assert!(candidates.iter().any(|c| c.replacement() == "u"));
        assert!(candidates.iter().any(|c| c.replacement() == "o"));
        assert!(candidates.iter().any(|c| c.replacement() == "users"));
        assert!(candidates.iter().any(|c| c.replacement() == "orders"));
    }

    #[test]
    fn test_completion_after_group_by() {
        let metadata = create_mock_metadata();
        let completer = ReplCompleter::new(metadata);
        let (start, candidates) = completer
            .complete(
                "SELECT COUNT(*) FROM users GROUP BY ",
                36,
                &Context::new(&DefaultHistory::new()),
            )
            .expect("valid completion call");

        assert_eq!(start, 36);
        assert!(candidates.iter().any(|c| c.replacement() == "id"));
        assert!(candidates.iter().any(|c| c.replacement() == "name"));
        assert!(candidates.iter().any(|c| c.replacement() == "email"));
    }

    #[test]
    fn test_completion_after_order_by() {
        let metadata = create_mock_metadata();
        let completer = ReplCompleter::new(metadata);
        let (start, candidates) = completer
            .complete(
                "SELECT * FROM users ORDER BY ",
                29,
                &Context::new(&DefaultHistory::new()),
            )
            .expect("valid completion call");

        assert_eq!(start, 29);
        assert!(candidates.iter().any(|c| c.replacement() == "id"));
        assert!(candidates.iter().any(|c| c.replacement() == "name"));
        assert!(candidates.iter().any(|c| c.replacement() == "email"));
    }

    #[test]
    fn test_completion_after_from() {
        let metadata = create_mock_metadata();
        let completer = ReplCompleter::new(metadata);
        let (start, candidates) = completer
            .complete("SELECT * FROM ", 14, &Context::new(&DefaultHistory::new()))
            .expect("valid completion call");

        assert_eq!(start, 14);
        assert!(candidates.iter().any(|c| c.replacement() == "users"));
        assert!(candidates.iter().any(|c| c.replacement() == "orders"));
    }

    fn create_mock_metadata() -> Metadata {
        let mut schema = Schema::new("public", true);
        let mut users_table = Table::new("users");
        users_table.add_column(Column::new("id", "INT", false, None));
        users_table.add_column(Column::new("name", "VARCHAR", true, Some("")));
        users_table.add_column(Column::new("email", "VARCHAR", true, Some("")));

        let mut orders_table = Table::new("orders");
        orders_table.add_column(Column::new("id", "INT", false, None));
        orders_table.add_column(Column::new("user_id", "INT", true, None));
        orders_table.add_column(Column::new("total", "FLOAT", true, None));
        schema.add(users_table);
        schema.add(orders_table);

        let mut metadata = Metadata::new();
        metadata.add(schema);
        metadata
    }

    #[test]
    fn test_find_previous_keyword() {
        let sql = "SELECT * FROM orders o JOIN users u ON o.user_id = u.id WHERE u.id = 7 AND o.total > 10.0";
        let tokens = Tokenizer::new(&GenericDialect, sql)
            .tokenize_with_location()
            .expect("valid sql");
        let mut index = tokens.len() - 1;

        let mut keywords_found: Vec<&TokenWithLocation> = vec![];
        while let Some((new_index, last_token)) = find_previous_keyword(&tokens, index) {
            index = new_index;
            keywords_found.push(last_token);
        }
        let expected = [
            Keyword::AND,
            Keyword::WHERE,
            Keyword::ON,
            Keyword::JOIN,
            Keyword::FROM,
            Keyword::SELECT,
        ];
        assert_eq!(keywords_found.len(), expected.len());

        keywords_found
            .iter()
            .zip(expected.iter())
            .for_each(|(found, expectation)| match found.token {
                Token::Word(Word {
                    keyword: found_keyword,
                    ..
                }) => assert_eq!(found_keyword, *expectation),
                _ => panic!("response from find_previous_keyword was not a keyword"),
            });
    }

    #[test]
    fn test_suggestion_default() {
        assert!(matches!(Suggestion::default(), Suggestion::Keyword(s) if s == String::new()));
    }
}
