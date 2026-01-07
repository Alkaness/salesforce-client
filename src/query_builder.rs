//! Type-safe SOQL query builder
//!
//! Provides a fluent API for constructing SOQL queries with compile-time guarantees.

use std::marker::PhantomData;

/// Type-safe SOQL query builder
///
/// # Example
/// ```
/// use salesforce_client::QueryBuilder;
///
/// let query = QueryBuilder::select(&["Id", "Name"])
///     .from("Account")
///     .where_clause("AnnualRevenue > 1000000")
///     .order_by("Name")
///     .limit(10)
///     .build();
///
/// assert_eq!(query, "SELECT Id, Name FROM Account WHERE AnnualRevenue > 1000000 ORDER BY Name LIMIT 10");
/// ```
#[derive(Debug, Clone)]
pub struct QueryBuilder<State = NeedsFrom> {
    fields: Vec<String>,
    from: Option<String>,
    where_clauses: Vec<String>,
    order_by: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
    _state: PhantomData<State>,
}

// Type states for compile-time query validation
#[derive(Debug, Clone)]
pub struct NeedsFrom;

#[derive(Debug, Clone)]
pub struct Complete;

impl QueryBuilder<NeedsFrom> {
    /// Start building a query with SELECT fields
    pub fn select(fields: &[&str]) -> Self {
        Self {
            fields: fields.iter().map(|s| s.to_string()).collect(),
            from: None,
            where_clauses: Vec::new(),
            order_by: None,
            limit: None,
            offset: None,
            _state: PhantomData,
        }
    }

    /// Specify the FROM clause (required)
    pub fn from(mut self, sobject: impl Into<String>) -> QueryBuilder<Complete> {
        self.from = Some(sobject.into());
        QueryBuilder {
            fields: self.fields,
            from: self.from,
            where_clauses: self.where_clauses,
            order_by: self.order_by,
            limit: self.limit,
            offset: self.offset,
            _state: PhantomData,
        }
    }
}

impl QueryBuilder<Complete> {
    /// Add a WHERE clause
    pub fn where_clause(mut self, condition: impl Into<String>) -> Self {
        self.where_clauses.push(condition.into());
        self
    }

    /// Add an AND condition to WHERE clause
    pub fn and(mut self, condition: impl Into<String>) -> Self {
        self.where_clauses.push(condition.into());
        self
    }

    /// Add an ORDER BY clause
    pub fn order_by(mut self, field: impl Into<String>) -> Self {
        self.order_by = Some(field.into());
        self
    }

    /// Add ORDER BY with direction
    pub fn order_by_asc(mut self, field: impl Into<String>) -> Self {
        self.order_by = Some(format!("{} ASC", field.into()));
        self
    }

    /// Add ORDER BY descending
    pub fn order_by_desc(mut self, field: impl Into<String>) -> Self {
        self.order_by = Some(format!("{} DESC", field.into()));
        self
    }

    /// Add a LIMIT clause
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Add an OFFSET clause
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Build the final SOQL query string
    pub fn build(self) -> String {
        let mut query = format!(
            "SELECT {} FROM {}",
            self.fields.join(", "),
            self.from.unwrap() // Safe because Complete state guarantees from is set
        );

        if !self.where_clauses.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&self.where_clauses.join(" AND "));
        }

        if let Some(order) = self.order_by {
            query.push_str(" ORDER BY ");
            query.push_str(&order);
        }

        if let Some(limit) = self.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        query
    }
}

/// Fluent API for building COUNT queries
pub struct CountQueryBuilder {
    from: String,
    where_clauses: Vec<String>,
}

impl CountQueryBuilder {
    /// Start building a COUNT query
    pub fn count_from(sobject: impl Into<String>) -> Self {
        Self {
            from: sobject.into(),
            where_clauses: Vec::new(),
        }
    }

    /// Add a WHERE clause
    pub fn where_clause(mut self, condition: impl Into<String>) -> Self {
        self.where_clauses.push(condition.into());
        self
    }

    /// Build the query
    pub fn build(self) -> String {
        let mut query = format!("SELECT COUNT() FROM {}", self.from);

        if !self.where_clauses.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&self.where_clauses.join(" AND "));
        }

        query
    }
}

/// Helper for building subqueries
pub struct SubqueryBuilder {
    fields: Vec<String>,
    relationship: String,
    where_clauses: Vec<String>,
    order_by: Option<String>,
    limit: Option<u32>,
}

impl SubqueryBuilder {
    /// Create a new subquery builder
    pub fn new(relationship: impl Into<String>, fields: &[&str]) -> Self {
        Self {
            fields: fields.iter().map(|s| s.to_string()).collect(),
            relationship: relationship.into(),
            where_clauses: Vec::new(),
            order_by: None,
            limit: None,
        }
    }

    /// Add a WHERE clause
    pub fn where_clause(mut self, condition: impl Into<String>) -> Self {
        self.where_clauses.push(condition.into());
        self
    }

    /// Add an ORDER BY clause
    pub fn order_by(mut self, field: impl Into<String>) -> Self {
        self.order_by = Some(field.into());
        self
    }

    /// Add a LIMIT clause
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Build the subquery string (for use in parent query)
    pub fn build(self) -> String {
        let mut query = format!(
            "(SELECT {} FROM {}",
            self.fields.join(", "),
            self.relationship
        );

        if !self.where_clauses.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&self.where_clauses.join(" AND "));
        }

        if let Some(order) = self.order_by {
            query.push_str(" ORDER BY ");
            query.push_str(&order);
        }

        if let Some(limit) = self.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        query.push(')');
        query
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_query() {
        let query = QueryBuilder::select(&["Id", "Name"])
            .from("Account")
            .build();

        assert_eq!(query, "SELECT Id, Name FROM Account");
    }

    #[test]
    fn test_query_with_where() {
        let query = QueryBuilder::select(&["Id", "Name"])
            .from("Account")
            .where_clause("AnnualRevenue > 1000000")
            .build();

        assert_eq!(
            query,
            "SELECT Id, Name FROM Account WHERE AnnualRevenue > 1000000"
        );
    }

    #[test]
    fn test_query_with_multiple_conditions() {
        let query = QueryBuilder::select(&["Id", "Name"])
            .from("Account")
            .where_clause("AnnualRevenue > 1000000")
            .and("Industry = 'Technology'")
            .build();

        assert_eq!(
            query,
            "SELECT Id, Name FROM Account WHERE AnnualRevenue > 1000000 AND Industry = 'Technology'"
        );
    }

    #[test]
    fn test_query_with_order_and_limit() {
        let query = QueryBuilder::select(&["Id", "Name"])
            .from("Account")
            .order_by("Name")
            .limit(10)
            .build();

        assert_eq!(query, "SELECT Id, Name FROM Account ORDER BY Name LIMIT 10");
    }

    #[test]
    fn test_query_with_all_clauses() {
        let query = QueryBuilder::select(&["Id", "Name", "AnnualRevenue"])
            .from("Account")
            .where_clause("AnnualRevenue > 1000000")
            .and("Industry = 'Technology'")
            .order_by_desc("AnnualRevenue")
            .limit(10)
            .offset(5)
            .build();

        assert_eq!(
            query,
            "SELECT Id, Name, AnnualRevenue FROM Account WHERE AnnualRevenue > 1000000 AND Industry = 'Technology' ORDER BY AnnualRevenue DESC LIMIT 10 OFFSET 5"
        );
    }

    #[test]
    fn test_count_query() {
        let query = CountQueryBuilder::count_from("Account")
            .where_clause("AnnualRevenue > 1000000")
            .build();

        assert_eq!(
            query,
            "SELECT COUNT() FROM Account WHERE AnnualRevenue > 1000000"
        );
    }

    #[test]
    fn test_subquery() {
        let subquery = SubqueryBuilder::new("Contacts", &["Id", "Email"])
            .where_clause("Email != null")
            .limit(5)
            .build();

        assert_eq!(
            subquery,
            "(SELECT Id, Email FROM Contacts WHERE Email != null LIMIT 5)"
        );
    }
}
