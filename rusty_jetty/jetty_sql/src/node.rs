use anyhow::Result;
use sqlparser::ast;

/// A wrapper for sqlparser::ast types that allows them to implement
/// the Teraversable trait
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) enum Node {
    Array(ast::Array),
    Assignment(ast::Assignment),
    ColumnDef(ast::ColumnDef),
    ColumnOptionDef(ast::ColumnOptionDef),
    Cte(ast::Cte),
    Fetch(ast::Fetch),
    Function(ast::Function),
    HiveFormat(ast::HiveFormat),
    Ident(ast::Ident),
    Join(ast::Join),
    LateralView(ast::LateralView),
    ListAgg(ast::ListAgg),
    ObjectName(ast::ObjectName),
    Offset(ast::Offset),
    OrderByExpr(ast::OrderByExpr),
    Query(ast::Query),
    Select(ast::Select),
    SelectInto(ast::SelectInto),
    SqlOption(ast::SqlOption),
    TableAlias(ast::TableAlias),
    TableWithJoins(ast::TableWithJoins),
    Top(ast::Top),
    Values(ast::Values),
    WindowFrame(ast::WindowFrame),
    WindowSpec(ast::WindowSpec),
    With(ast::With),
    Action(ast::Action),
    AddDropSync(ast::AddDropSync),
    AlterColumnOperation(ast::AlterColumnOperation),
    AlterTableOperation(ast::AlterTableOperation),
    BinaryOperator(ast::BinaryOperator),
    CloseCursor(ast::CloseCursor),
    ColumnOption(ast::ColumnOption),
    CommentObject(ast::CommentObject),
    CopyLegacyCsvOption(ast::CopyLegacyCsvOption),
    CopyLegacyOption(ast::CopyLegacyOption),
    CopyOption(ast::CopyOption),
    CopyTarget(ast::CopyTarget),
    CreateFunctionUsing(ast::CreateFunctionUsing),
    DataType(ast::DataType),
    DateTimeField(ast::DateTimeField),
    DiscardObject(ast::DiscardObject),
    Expr(ast::Expr),
    FetchDirection(ast::FetchDirection),
    FileFormat(ast::FileFormat),
    FunctionArg(ast::FunctionArg),
    FunctionArgExpr(ast::FunctionArgExpr),
    GrantObjects(ast::GrantObjects),
    HiveDistributionStyle(ast::HiveDistributionStyle),
    HiveIOFormat(ast::HiveIOFormat),
    HiveRowFormat(ast::HiveRowFormat),
    JoinConstraint(ast::JoinConstraint),
    JoinOperator(ast::JoinOperator),
    JsonOperator(ast::JsonOperator),
    KillType(ast::KillType),
    ListAggOnOverflow(ast::ListAggOnOverflow),
    LockType(ast::LockType),
    MergeClause(ast::MergeClause),
    ObjectType(ast::ObjectType),
    OffsetRows(ast::OffsetRows),
    OnCommit(ast::OnCommit),
    OnInsert(ast::OnInsert),
    Privileges(ast::Privileges),
    ReferentialAction(ast::ReferentialAction),
    SelectItem(ast::SelectItem),
    SetExpr(ast::SetExpr),
    SetOperator(ast::SetOperator),
    ShowCreateObject(ast::ShowCreateObject),
    ShowStatementFilter(ast::ShowStatementFilter),
    SqliteOnConflict(ast::SqliteOnConflict),
    Statement(ast::Statement),
    TableConstraint(ast::TableConstraint),
    TableFactor(ast::TableFactor),
    TransactionAccessMode(ast::TransactionAccessMode),
    TransactionIsolationLevel(ast::TransactionIsolationLevel),
    TransactionMode(ast::TransactionMode),
    TrimWhereField(ast::TrimWhereField),
    UnaryOperator(ast::UnaryOperator),
    Value(ast::Value),
    WindowFrameBound(ast::WindowFrameBound),
    WindowFrameUnits(ast::WindowFrameUnits),
}

/// This trait is implemented by AST types and returns child ast nodes.
/// It does not include terminal nodes (such as boolean or text values)
pub(crate) trait Traversable {
    fn get_children(&self) -> Result<Vec<Node>>;
}

impl Node {
    /// This function runs a a depth-first traversal and accumulation of the descendent nodes
    pub(crate) fn get_descendants(&self) -> Result<Vec<Node>> {
        let mut d = Vec::new();
        for child in self.get_children()? {
            d.push(child.to_owned());
            d.extend(child.get_descendants()?)
        }
        Ok(d)
    }
}

/// This Macro implements the Traversable trait for node by matching on
/// the inner enum types
macro_rules! impl_traversable_node {
    ($($t:tt),+) => {
        impl Traversable for Node {
            fn get_children(&self) -> Result<Vec<Node>> {
                match self {
                    $(Node::$t(n) => n.get_children(),)*
                    _ => panic!("Not supported. Please insert another quarter. {:#?}", &self),
                }
            }
        }
    }
}

impl_traversable_node!(
    Fetch,
    Ident,
    Join,
    LateralView,
    ObjectName,
    Offset,
    OrderByExpr,
    Query,
    Select,
    SelectInto,
    TableAlias,
    TableWithJoins,
    Top,
    Values,
    With,
    Expr,
    FunctionArg,
    LockType,
    SelectItem,
    SetExpr,
    SetOperator,
    Statement,
    TableFactor,
    Value
);

impl Traversable for ast::Array {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}

impl Traversable for ast::Assignment {
    fn get_children(&self) -> Result<Vec<Node>> {
        let mut children: Vec<Node> = self
            .id
            .iter()
            .map(|id| Node::Ident(id.to_owned()))
            .collect();
        children.extend([Node::Expr(self.value.to_owned())]);
        Ok(children)
    }
}
impl Traversable for ast::ColumnDef {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::ColumnOptionDef {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::Cte {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::Fetch {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::Function {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::HiveFormat {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::Ident {
    fn get_children(&self) -> Result<Vec<Node>> {
        Ok(Vec::new())
    }
}
impl Traversable for ast::Join {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::LateralView {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::ListAgg {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::ObjectName {
    fn get_children(&self) -> Result<Vec<Node>> {
        Ok(self.0.iter().map(|n| Node::Ident(n.to_owned())).collect())
    }
}
impl Traversable for ast::Offset {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::OrderByExpr {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::Query {
    fn get_children(&self) -> Result<Vec<Node>> {
        let mut children = Vec::new();
        if let Some(with) = &self.with {
            children.push(Node::With(with.to_owned()))
        }

        children.push(Node::SetExpr(*self.body.to_owned()));

        children.extend(
            self.order_by
                .iter()
                .map(|e| Node::OrderByExpr(e.to_owned())),
        );

        if let Some(node) = &self.limit {
            children.push(Node::Expr(node.to_owned()))
        }
        if let Some(node) = &self.offset {
            children.push(Node::Offset(node.to_owned()))
        }
        if let Some(node) = &self.fetch {
            children.push(Node::Fetch(node.to_owned()))
        }
        if let Some(node) = &self.lock {
            children.push(Node::LockType(node.to_owned()))
        };
        Ok(children)
    }
}
impl Traversable for ast::Select {
    fn get_children(&self) -> Result<Vec<Node>> {
        let mut children = Vec::new();
        if let Some(top) = &self.top {
            children.push(Node::Top(top.to_owned()))
        }
        children.extend(
            self.projection
                .iter()
                .map(|n| Node::SelectItem(n.to_owned())),
        );
        if let Some(n) = &self.into {
            children.push(Node::SelectInto(n.to_owned()))
        };
        children.extend(self.from.iter().map(|n| Node::TableWithJoins(n.to_owned())));
        children.extend(
            self.lateral_views
                .iter()
                .map(|n| Node::LateralView(n.to_owned())),
        );
        if let Some(n) = &self.selection {
            children.push(Node::Expr(n.to_owned()))
        };
        children.extend(self.group_by.iter().map(|n| Node::Expr(n.to_owned())));
        children.extend(self.cluster_by.iter().map(|n| Node::Expr(n.to_owned())));
        children.extend(self.distribute_by.iter().map(|n| Node::Expr(n.to_owned())));
        children.extend(self.sort_by.iter().map(|n| Node::Expr(n.to_owned())));
        if let Some(n) = &self.having {
            children.push(Node::Expr(n.to_owned()))
        };
        if let Some(n) = &self.qualify {
            children.push(Node::Expr(n.to_owned()))
        };
        Ok(children)
    }
}
impl Traversable for ast::SelectInto {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::SqlOption {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::TableAlias {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::TableWithJoins {
    fn get_children(&self) -> Result<Vec<Node>> {
        let mut children = vec![Node::TableFactor(self.relation.to_owned())];
        children.extend(self.joins.iter().map(|n| Node::Join(n.to_owned())));
        Ok(children)
    }
}
impl Traversable for ast::Top {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::Values {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::WindowFrame {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::WindowSpec {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::With {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::Action {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::AddDropSync {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::AlterColumnOperation {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::AlterTableOperation {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::BinaryOperator {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::CloseCursor {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::ColumnOption {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::CommentObject {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::CopyLegacyCsvOption {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::CopyLegacyOption {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::CopyOption {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::CopyTarget {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::CreateFunctionUsing {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::DataType {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::DateTimeField {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::DiscardObject {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::Expr {
    fn get_children(&self) -> Result<Vec<Node>> {
        dbg!(&self);
        Ok(match self {
            ast::Expr::Identifier(n) => vec![Node::Ident(n.to_owned())],
            ast::Expr::CompoundIdentifier(_) => todo!(),
            ast::Expr::JsonAccess {
                left,
                operator,
                right,
            } => todo!(),
            ast::Expr::CompositeAccess { expr, key } => todo!(),
            ast::Expr::IsFalse(_) => todo!(),
            ast::Expr::IsNotFalse(_) => todo!(),
            ast::Expr::IsTrue(_) => todo!(),
            ast::Expr::IsNotTrue(_) => todo!(),
            ast::Expr::IsNull(_) => todo!(),
            ast::Expr::IsNotNull(_) => todo!(),
            ast::Expr::IsUnknown(_) => todo!(),
            ast::Expr::IsNotUnknown(_) => todo!(),
            ast::Expr::IsDistinctFrom(_, _) => todo!(),
            ast::Expr::IsNotDistinctFrom(_, _) => todo!(),
            ast::Expr::InList {
                expr,
                list,
                negated,
            } => todo!(),
            ast::Expr::InSubquery {
                expr,
                subquery,
                negated,
            } => todo!(),
            ast::Expr::InUnnest {
                expr,
                array_expr,
                negated,
            } => todo!(),
            ast::Expr::Between {
                expr,
                negated,
                low,
                high,
            } => todo!(),
            ast::Expr::BinaryOp { left, op, right } => todo!(),
            ast::Expr::Like {
                negated,
                expr,
                pattern,
                escape_char,
            } => todo!(),
            ast::Expr::ILike {
                negated,
                expr,
                pattern,
                escape_char,
            } => todo!(),
            ast::Expr::SimilarTo {
                negated,
                expr,
                pattern,
                escape_char,
            } => todo!(),
            ast::Expr::AnyOp(_) => todo!(),
            ast::Expr::AllOp(_) => todo!(),
            ast::Expr::UnaryOp { op, expr } => todo!(),
            ast::Expr::Cast { expr, data_type } => todo!(),
            ast::Expr::TryCast { expr, data_type } => todo!(),
            ast::Expr::SafeCast { expr, data_type } => todo!(),
            ast::Expr::AtTimeZone {
                timestamp,
                time_zone,
            } => todo!(),
            ast::Expr::Extract { field, expr } => todo!(),
            ast::Expr::Position { expr, r#in } => todo!(),
            ast::Expr::Substring {
                expr,
                substring_from,
                substring_for,
            } => todo!(),
            ast::Expr::Trim {
                expr,
                trim_where,
                trim_what,
            } => todo!(),
            ast::Expr::Overlay {
                expr,
                overlay_what,
                overlay_from,
                overlay_for,
            } => todo!(),
            ast::Expr::Collate { expr, collation } => todo!(),
            ast::Expr::Nested(_) => todo!(),
            ast::Expr::Value(n) => vec![Node::Value(n.to_owned())],
            ast::Expr::TypedString { data_type, value } => todo!(),
            ast::Expr::MapAccess { column, keys } => todo!(),
            ast::Expr::Function(_) => todo!(),
            ast::Expr::Case {
                operand,
                conditions,
                results,
                else_result,
            } => todo!(),
            ast::Expr::Exists { subquery, negated } => todo!(),
            ast::Expr::Subquery(_) => todo!(),
            ast::Expr::ArraySubquery(_) => todo!(),
            ast::Expr::ListAgg(_) => todo!(),
            ast::Expr::GroupingSets(_) => todo!(),
            ast::Expr::Cube(_) => todo!(),
            ast::Expr::Rollup(_) => todo!(),
            ast::Expr::Tuple(_) => todo!(),
            ast::Expr::ArrayIndex { obj, indexes } => todo!(),
            ast::Expr::Array(_) => todo!(),
        })
    }
}
impl Traversable for ast::FetchDirection {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::FileFormat {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::FunctionArg {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::FunctionArgExpr {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::GrantObjects {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::HiveDistributionStyle {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::HiveIOFormat {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::HiveRowFormat {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::JoinConstraint {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::JoinOperator {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::JsonOperator {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::KillType {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::ListAggOnOverflow {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::LockType {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::MergeClause {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::ObjectType {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::OffsetRows {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::OnCommit {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::OnInsert {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::Privileges {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::ReferentialAction {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::SelectItem {
    fn get_children(&self) -> Result<Vec<Node>> {
        let children = match self {
            ast::SelectItem::UnnamedExpr(n) => vec![Node::Expr(n.to_owned())],
            ast::SelectItem::ExprWithAlias { expr, alias } => {
                vec![Node::Expr(expr.to_owned()), Node::Ident(alias.to_owned())]
            }
            ast::SelectItem::QualifiedWildcard(n) => vec![Node::ObjectName(n.to_owned())],
            ast::SelectItem::Wildcard => vec![],
        };
        Ok(children)
    }
}
impl Traversable for ast::SetExpr {
    fn get_children(&self) -> Result<Vec<Node>> {
        let children = match self {
            ast::SetExpr::Select(n) => vec![Node::Select(*n.to_owned())],
            ast::SetExpr::Query(n) => vec![Node::Query(*n.to_owned())],
            ast::SetExpr::SetOperation {
                op,
                all: _,
                left,
                right,
            } => {
                vec![
                    Node::SetOperator(op.to_owned()),
                    Node::SetExpr(*left.to_owned()),
                    Node::SetExpr(*right.to_owned()),
                ]
            }
            ast::SetExpr::Values(n) => vec![Node::Values(n.to_owned())],
            ast::SetExpr::Insert(n) => vec![Node::Statement(n.to_owned())],
        };
        Ok(children)
    }
}
impl Traversable for ast::SetOperator {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::ShowCreateObject {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::ShowStatementFilter {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::SqliteOnConflict {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::Statement {
    fn get_children(&self) -> Result<Vec<Node>> {
        let children = match self {
            ast::Statement::Query(q) => {
                vec![Node::Query(*q.to_owned())]
            }
            _ => todo!(),
        };
        Ok(children)
    }
}
impl Traversable for ast::TableConstraint {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::TableFactor {
    fn get_children(&self) -> Result<Vec<Node>> {
        match self {
            ast::TableFactor::Table {
                name,
                alias,
                args,
                with_hints,
            } => {
                let mut children = vec![Node::ObjectName(name.to_owned())];
                if let Some(n) = alias {
                    children.push(Node::TableAlias(n.to_owned()))
                };
                if let Some(n) = args {
                    children.extend(n.iter().map(|n| Node::FunctionArg(n.to_owned())));
                };
                children.extend(with_hints.iter().map(|n| Node::Expr(n.to_owned())));

                Ok(children)
            }
            ast::TableFactor::Derived {
                lateral: _,
                subquery,
                alias,
            } => {
                let mut children = vec![(Node::Query(*subquery.to_owned()))];
                if let Some(n) = alias {
                    children.push(Node::TableAlias(n.to_owned()));
                };
                Ok(children)
            }
            ast::TableFactor::TableFunction { expr, alias } => {
                let mut children = vec![Node::Expr(expr.to_owned())];
                if let Some(n) = alias {
                    children.push(Node::TableAlias(n.to_owned()));
                };

                Ok(children)
            }

            ast::TableFactor::UNNEST {
                alias,
                array_expr,
                with_offset: _,
                with_offset_alias,
            } => {
                let mut children = Vec::new();
                if let Some(n) = alias {
                    children.push(Node::TableAlias(n.to_owned()));
                };
                children.push(Node::Expr(*array_expr.to_owned()));
                if let Some(n) = with_offset_alias {
                    children.push(Node::Ident(n.to_owned()));
                };
                Ok(children)
            }
            ast::TableFactor::NestedJoin {
                table_with_joins,
                alias,
            } => {
                let mut children = vec![Node::TableWithJoins(*table_with_joins.to_owned())];
                if let Some(n) = alias {
                    children.push(Node::TableAlias(n.to_owned()));
                };
                Ok(children)
            }
        }
    }
}
impl Traversable for ast::TransactionAccessMode {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::TransactionIsolationLevel {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::TransactionMode {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::TrimWhereField {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::UnaryOperator {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::Value {
    fn get_children(&self) -> Result<Vec<Node>> {
        Ok(match self {
            ast::Value::Number(_, _) => vec![],
            ast::Value::SingleQuotedString(_) => todo!(),
            ast::Value::EscapedStringLiteral(_) => todo!(),
            ast::Value::NationalStringLiteral(_) => todo!(),
            ast::Value::HexStringLiteral(_) => todo!(),
            ast::Value::DoubleQuotedString(_) => todo!(),
            ast::Value::Boolean(_) => todo!(),
            ast::Value::Interval {
                value,
                leading_field,
                leading_precision,
                last_field,
                fractional_seconds_precision,
            } => todo!(),
            ast::Value::Null => todo!(),
            ast::Value::Placeholder(_) => todo!(),
        })
    }
}
impl Traversable for ast::WindowFrameBound {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
impl Traversable for ast::WindowFrameUnits {
    fn get_children(&self) -> Result<Vec<Node>> {
        todo!()
    }
}
