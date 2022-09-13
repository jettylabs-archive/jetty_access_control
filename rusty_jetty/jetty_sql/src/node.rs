use sqlparser::{ast, keywords::NO};

enum Node {
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
    Bool(bool),
}

trait Traversable {
    fn get_children(&self) -> Vec<Node>;
}

impl Traversable for ast::Array {
    fn get_children(&self) -> Vec<Node> {
        self.elem.iter().map(|e| Node::Expr(e.to_owned())).collect()
    }
}

impl Traversable for ast::Assignment {
    fn get_children(&self) -> Vec<Node> {
        let mut children: Vec<Node> = self
            .id
            .iter()
            .map(|id| Node::Ident(id.to_owned()))
            .collect();
        children.extend([Node::Expr(self.value.to_owned())]);
        children
    }
}
impl Traversable for ast::ColumnDef {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::ColumnOptionDef {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::Cte {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::Fetch {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::Function {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::HiveFormat {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::Ident {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::Join {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::LateralView {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::ListAgg {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::ObjectName {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::Offset {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::OrderByExpr {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::Query {
    fn get_children(&self) -> Vec<Node> {
        let mut children = Vec::new();
        if let Some(with) = self.with {
            children.push(Node::With(with.to_owned()))
        }

        children.push(Node::SetExpr(*self.body.to_owned()));

        children.extend(
            self.order_by
                .iter()
                .map(|&e| Node::OrderByExpr(e.to_owned())),
        );

        if let Some(node) = self.limit {
            children.push(Node::Expr(node.to_owned()))
        }
        if let Some(node) = self.offset {
            children.push(Node::Offset(node.to_owned()))
        }
        if let Some(node) = self.fetch {
            children.push(Node::Fetch(node.to_owned()))
        }
        if let Some(node) = self.lock {
            children.push(Node::LockType(node.to_owned()))
        }
        children
    }
}
impl Traversable for ast::Select {
    fn get_children(&self) -> Vec<Node> {
        let children = Vec::new();
        children.push(Node::Bool(self.distinct.to_owned()));
        if let Some(top) = self.top {
            children.push(Node::Top(top.to_owned()))
        }
        children.extend(
            self.projection
                .iter()
                .map(|n| Node::SelectItem(n.to_owned())),
        );
        if let Some(n) = self.into {
            children.push(Node::SelectInto(n.to_owned()))
        };
        children.extend(self.from.iter().map(|n| Node::TableWithJoins(n.to_owned())));
        children.extend(
            self.lateral_views
                .iter()
                .map(|n| Node::LateralView(n.to_owned())),
        );
        if let Some(n) = self.selection {
            children.push(Node::Expr(n.to_owned()))
        };
        children.extend(self.group_by.iter().map(|n| Node::Expr(n.to_owned())));
        children.extend(self.cluster_by.iter().map(|n| Node::Expr(n.to_owned())));
        children.extend(self.distribute_by.iter().map(|n| Node::Expr(n.to_owned())));
        children.extend(self.sort_by.iter().map(|n| Node::Expr(n.to_owned())));
        if let Some(n) = self.having {
            children.push(Node::Expr(n.to_owned()))
        };
        if let Some(n) = self.qualify {
            children.push(Node::Expr(n.to_owned()))
        };

        children
    }
}
impl Traversable for ast::SelectInto {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::SqlOption {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::TableAlias {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::TableWithJoins {
    fn get_children(&self) -> Vec<Node> {
        let mut children = vec![Node::TableFactor(self.relation)];
        children.extend(self.joins.iter().map(|n| Node::Join(n.to_owned())));
        children
    }
}
impl Traversable for ast::Top {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::Values {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::WindowFrame {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::WindowSpec {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::Action {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::AddDropSync {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::AlterColumnOperation {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::AlterTableOperation {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::BinaryOperator {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::CloseCursor {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::ColumnOption {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::CommentObject {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::CopyLegacyCsvOption {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::CopyLegacyOption {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::CopyOption {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::CopyTarget {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::CreateFunctionUsing {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::DataType {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::DateTimeField {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::DiscardObject {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::Expr {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::FetchDirection {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::FileFormat {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::FunctionArg {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::FunctionArgExpr {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::GrantObjects {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::HiveDistributionStyle {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::HiveIOFormat {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::HiveRowFormat {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::JoinConstraint {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::JoinOperator {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::JsonOperator {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::KillType {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::ListAggOnOverflow {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::LockType {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::MergeClause {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::ObjectType {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::OffsetRows {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::OnCommit {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::OnInsert {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::Privileges {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::ReferentialAction {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::SelectItem {
    fn get_children(&self) -> Vec<Node> {
        match self {
            ast::SelectItem::UnnamedExpr(n) => vec![Node::Expr(n.to_owned())],
            ast::SelectItem::ExprWithAlias { expr, alias } => {
                vec![Node::Expr(expr.to_owned()), Node::Ident(alias.to_owned())]
            }
            ast::SelectItem::QualifiedWildcard(n) => vec![Node::ObjectName(n.to_owned())],
            ast::SelectItem::Wildcard => vec![],
        }
    }
}
impl Traversable for ast::SetExpr {
    fn get_children(&self) -> Vec<Node> {
        match self {
            ast::SetExpr::Select(n) => vec![Node::Select(*n.to_owned())],
            ast::SetExpr::Query(n) => vec![Node::Query(*n.to_owned())],
            ast::SetExpr::SetOperation {
                op,
                all,
                left,
                right,
            } => {
                vec![
                    Node::SetOperator(op.to_owned()),
                    Node::Bool(all.to_owned()),
                    Node::SetExpr(*left.to_owned()),
                    Node::SetExpr(*right.to_owned()),
                ]
            }
            ast::SetExpr::Values(n) => vec![Node::Values(n.to_owned())],
            ast::SetExpr::Insert(n) => vec![Node::Statement(n.to_owned())],
        }
    }
}
impl Traversable for ast::SetOperator {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::ShowCreateObject {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::ShowStatementFilter {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::SqliteOnConflict {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::Statement {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::TableConstraint {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::TableFactor {
    fn get_children(&self) -> Vec<Node> {
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
                    children.extend(args.iter().map(|n| Node::FunctionArg(n.to_owned())));
                };
                children
            }
            ast::TableFactor::Derived {
                lateral,
                subquery,
                alias,
            } => todo!(),
            ast::TableFactor::TableFunction { expr, alias } => todo!(),
            ast::TableFactor::UNNEST {
                alias,
                array_expr,
                with_offset,
                with_offset_alias,
            } => todo!(),
            ast::TableFactor::NestedJoin {
                table_with_joins,
                alias,
            } => todo!(),
        }
    }
}
impl Traversable for ast::TransactionAccessMode {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::TransactionIsolationLevel {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::TransactionMode {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::TrimWhereField {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::UnaryOperator {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::Value {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::WindowFrameBound {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for ast::WindowFrameUnits {
    fn get_children(&self) -> Vec<Node> {
        todo!()
    }
}
impl Traversable for bool {
    fn get_children(&self) -> Vec<Node> {
        Vec::new()
    }
}
