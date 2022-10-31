use std::vec;

use sqlparser::ast;

/// Return a Vec<Node> from the variable and node type
macro_rules! value_child {
    ($value:ident, $node_type:tt) => {
        vec![Node::$node_type($value.to_owned())]
    };
}

/// Return a Vec<Node> from a Vec given vec and node type
macro_rules! vec_child {
    ($value:ident, $node_type:tt) => {
        $value
            .iter()
            .map(|n| Node::$node_type(n.to_owned()))
            .collect::<Vec<Node>>()
    };
}

/// Return a Vec<Node> of the specified type from an ast node wrapped in an option
macro_rules! option_child {
    ($value:ident, $node_type:tt) => {
        match $value {
            Some(n) => vec![Node::$node_type(n.to_owned())],
            None => vec![],
        }
    };
}

/// Return a Vec<Node> of the specified type from an ast node wrapped in a Box
macro_rules! box_child {
    ($value:ident, $node_type:tt) => {
        vec![Node::$node_type(*$value.to_owned())]
    };
}

/// Return a Vec<Node> of the specified type from an ast node wrapped in an option then a Box
macro_rules! option_box_child {
    ($value:ident, $node_type:tt) => {
        match $value {
            Some(n) => vec![Node::$node_type(*n.to_owned())],
            None => vec![],
        }
    };
}

/// Return a Vec<Node> of the specified type from an ast node wrapped in an option then a vec
macro_rules! option_vec_child {
    ($value:ident, $node_type:tt) => {
        match $value {
            Some(n) => n
                .iter()
                .map(|m| Node::$node_type(m.to_owned()))
                .collect::<Vec<Node>>(),
            None => vec![],
        }
    };
}

/// Return a Vec<Node> of the specified type from an ast node wrapped in nested vecs
macro_rules! vec_vec_child {
    ($value:ident, $node_type:tt) => {
        $value
            .iter()
            .flatten()
            .map(|n| Node::$node_type(n.to_owned()))
            .collect()
    };
}

/// A wrapper for sqlparser::ast types that allows them to implement
/// the Traversable trait
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
    fn get_children(&self) -> Vec<Node>;
}

impl Node {
    /// This function runs a a depth-first traversal and accumulation of the descendent nodes
    pub(crate) fn get_descendants(&self) -> Vec<Node> {
        let mut d = Vec::new();
        for child in self.get_children() {
            d.push(child.to_owned());
            d.extend(child.get_descendants())
        }
        d
    }
}

/// This Macro implements the Traversable trait for node by matching on
/// the inner enum types

macro_rules! impl_traversable_node {
    ($($t:tt),+) => {
        impl Traversable for Node {
            fn get_children(&self) -> Vec<Node> {
                #[allow(unreachable_patterns)]
                match self {
                    $(Node::$t(n) => n.get_children(),)*
                    _ => panic!("Not supported. Please insert another quarter. {:#?}", &self),
                }
            }
        }
    }
}

impl_traversable_node!(
    Array,
    Assignment,
    ColumnDef,
    ColumnOptionDef,
    Cte,
    Fetch,
    Function,
    HiveFormat,
    Ident,
    Join,
    LateralView,
    ListAgg,
    ObjectName,
    Offset,
    OrderByExpr,
    Query,
    Select,
    SelectInto,
    SqlOption,
    TableAlias,
    TableWithJoins,
    Top,
    Values,
    WindowFrame,
    WindowSpec,
    With,
    Action,
    AddDropSync,
    AlterColumnOperation,
    AlterTableOperation,
    BinaryOperator,
    CloseCursor,
    ColumnOption,
    CommentObject,
    CopyLegacyCsvOption,
    CopyLegacyOption,
    CopyOption,
    CopyTarget,
    CreateFunctionUsing,
    DataType,
    DateTimeField,
    DiscardObject,
    Expr,
    FetchDirection,
    FileFormat,
    FunctionArg,
    FunctionArgExpr,
    GrantObjects,
    HiveDistributionStyle,
    HiveIOFormat,
    HiveRowFormat,
    JoinConstraint,
    JoinOperator,
    JsonOperator,
    KillType,
    ListAggOnOverflow,
    LockType,
    MergeClause,
    ObjectType,
    OffsetRows,
    OnCommit,
    OnInsert,
    Privileges,
    ReferentialAction,
    SelectItem,
    SetExpr,
    SetOperator,
    ShowCreateObject,
    ShowStatementFilter,
    SqliteOnConflict,
    Statement,
    TableConstraint,
    TableFactor,
    TransactionAccessMode,
    TransactionIsolationLevel,
    TransactionMode,
    TrimWhereField,
    UnaryOperator,
    Value,
    WindowFrameBound,
    WindowFrameUnits
);

impl Traversable for ast::Array {
    fn get_children(&self) -> Vec<Node> {
        self.elem.iter().map(|n| Node::Expr(n.to_owned())).collect()
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
        [
            vec![
                Node::Ident(self.name.to_owned()),
                Node::DataType(self.data_type.to_owned()),
            ],
            match &self.collation {
                Some(n) => vec![Node::ObjectName(n.to_owned())],
                None => vec![],
            },
            self.options
                .iter()
                .map(|n| Node::ColumnOptionDef(n.to_owned()))
                .collect(),
        ]
        .concat()
    }
}
impl Traversable for ast::ColumnOptionDef {
    fn get_children(&self) -> Vec<Node> {
        [
            match &self.name {
                Some(n) => vec![Node::Ident(n.to_owned())],
                None => vec![],
            },
            vec![Node::ColumnOption(self.option.to_owned())],
        ]
        .concat()
    }
}
impl Traversable for ast::Cte {
    fn get_children(&self) -> Vec<Node> {
        [
            vec![
                Node::TableAlias(self.alias.to_owned()),
                Node::Query(self.query.to_owned()),
            ],
            match &self.from {
                Some(n) => vec![Node::Ident(n.to_owned())],
                None => vec![],
            },
        ]
        .concat()
    }
}
impl Traversable for ast::Fetch {
    fn get_children(&self) -> Vec<Node> {
        match &self.quantity {
            Some(n) => vec![Node::Expr(n.to_owned())],
            None => vec![],
        }
    }
}
impl Traversable for ast::Function {
    fn get_children(&self) -> Vec<Node> {
        [
            vec![Node::ObjectName(self.name.to_owned())],
            self.args
                .iter()
                .map(|n| Node::FunctionArg(n.to_owned()))
                .collect(),
            match &self.over {
                Some(n) => vec![Node::WindowSpec(n.to_owned())],
                None => vec![],
            },
        ]
        .concat()
    }
}
impl Traversable for ast::HiveFormat {
    fn get_children(&self) -> Vec<Node> {
        [
            match &self.row_format {
                Some(n) => vec![Node::HiveRowFormat(n.to_owned())],
                None => vec![],
            },
            match &self.storage {
                Some(n) => vec![Node::HiveIOFormat(n.to_owned())],
                None => vec![],
            },
        ]
        .concat()
    }
}
impl Traversable for ast::Ident {
    fn get_children(&self) -> Vec<Node> {
        Vec::new()
    }
}
impl Traversable for ast::Join {
    fn get_children(&self) -> Vec<Node> {
        vec![
            Node::TableFactor(self.relation.to_owned()),
            Node::JoinOperator(self.join_operator.to_owned()),
        ]
    }
}
impl Traversable for ast::LateralView {
    fn get_children(&self) -> Vec<Node> {
        [
            vec![
                Node::Expr(self.lateral_view.to_owned()),
                Node::ObjectName(self.lateral_view_name.to_owned()),
            ],
            self.lateral_col_alias
                .iter()
                .map(|n| Node::Ident(n.to_owned()))
                .collect(),
        ]
        .concat()
    }
}
impl Traversable for ast::ListAgg {
    fn get_children(&self) -> Vec<Node> {
        [
            vec![Node::Expr(*self.expr.to_owned())],
            match &self.separator {
                Some(n) => vec![Node::Expr(*n.to_owned())],
                None => vec![],
            },
            match &self.on_overflow {
                Some(n) => vec![Node::ListAggOnOverflow(n.to_owned())],
                None => vec![],
            },
            self.within_group
                .iter()
                .map(|n| Node::OrderByExpr(n.to_owned()))
                .collect(),
        ]
        .concat()
    }
}
impl Traversable for ast::ObjectName {
    fn get_children(&self) -> Vec<Node> {
        self.0.iter().map(|n| Node::Ident(n.to_owned())).collect()
    }
}
impl Traversable for ast::Offset {
    fn get_children(&self) -> Vec<Node> {
        vec![
            Node::Expr(self.value.to_owned()),
            Node::OffsetRows(self.rows.to_owned()),
        ]
    }
}
impl Traversable for ast::OrderByExpr {
    fn get_children(&self) -> Vec<Node> {
        vec![Node::Expr(self.expr.to_owned())]
    }
}
impl Traversable for ast::Query {
    fn get_children(&self) -> Vec<Node> {
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
        children
    }
}
impl Traversable for ast::Select {
    fn get_children(&self) -> Vec<Node> {
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
        children
    }
}
impl Traversable for ast::SelectInto {
    fn get_children(&self) -> Vec<Node> {
        vec![Node::ObjectName(self.name.to_owned())]
    }
}
impl Traversable for ast::SqlOption {
    fn get_children(&self) -> Vec<Node> {
        vec![
            Node::Ident(self.name.to_owned()),
            Node::Value(self.value.to_owned()),
        ]
    }
}
impl Traversable for ast::TableAlias {
    fn get_children(&self) -> Vec<Node> {
        [
            vec![Node::Ident(self.name.to_owned())],
            self.columns
                .iter()
                .map(|n| Node::Ident(n.to_owned()))
                .collect(),
        ]
        .concat()
    }
}
impl Traversable for ast::TableWithJoins {
    fn get_children(&self) -> Vec<Node> {
        let mut children = vec![Node::TableFactor(self.relation.to_owned())];
        children.extend(self.joins.iter().map(|n| Node::Join(n.to_owned())));
        children
    }
}
impl Traversable for ast::Top {
    fn get_children(&self) -> Vec<Node> {
        match &self.quantity {
            Some(n) => vec![Node::Expr(n.to_owned())],
            None => vec![],
        }
    }
}
impl Traversable for ast::Values {
    fn get_children(&self) -> Vec<Node> {
        self.0
            .iter()
            .flatten()
            .map(|n| Node::Expr(n.to_owned()))
            .collect()
    }
}
impl Traversable for ast::WindowFrame {
    fn get_children(&self) -> Vec<Node> {
        [
            vec![
                Node::WindowFrameUnits(self.units.to_owned()),
                Node::WindowFrameBound(self.start_bound.to_owned()),
            ],
            match &self.end_bound {
                Some(n) => vec![Node::WindowFrameBound(n.to_owned())],
                None => vec![],
            },
        ]
        .concat()
    }
}
impl Traversable for ast::WindowSpec {
    fn get_children(&self) -> Vec<Node> {
        [
            self.partition_by
                .iter()
                .map(|n| Node::Expr(n.to_owned()))
                .collect(),
            self.order_by
                .iter()
                .map(|n| Node::OrderByExpr(n.to_owned()))
                .collect(),
            match &self.window_frame {
                Some(n) => vec![Node::WindowFrame(n.to_owned())],
                None => vec![],
            },
        ]
        .concat()
    }
}
impl Traversable for ast::With {
    fn get_children(&self) -> Vec<Node> {
        self.cte_tables
            .iter()
            .map(|n| Node::Cte(n.to_owned()))
            .collect()
    }
}
impl Traversable for ast::Action {
    fn get_children(&self) -> Vec<Node> {
        match &self {
            ast::Action::Insert { columns: Some(n) } => {
                n.iter().map(|n| Node::Ident(n.to_owned())).collect()
            }
            ast::Action::References { columns: Some(n) } => {
                n.iter().map(|n| Node::Ident(n.to_owned())).collect()
            }
            ast::Action::Select { columns: Some(n) } => {
                n.iter().map(|n| Node::Ident(n.to_owned())).collect()
            }
            ast::Action::Update { columns: Some(n) } => {
                n.iter().map(|n| Node::Ident(n.to_owned())).collect()
            }
            _ => vec![],
        }
    }
}
impl Traversable for ast::AddDropSync {
    fn get_children(&self) -> Vec<Node> {
        vec![]
    }
}
impl Traversable for ast::AlterColumnOperation {
    fn get_children(&self) -> Vec<Node> {
        match &self {
            ast::AlterColumnOperation::SetDefault { value } => {
                vec![Node::Expr(value.to_owned())]
            }
            ast::AlterColumnOperation::SetDataType { data_type, using } => [
                vec![Node::DataType(data_type.to_owned())],
                match using {
                    Some(n) => vec![Node::Expr(n.to_owned())],
                    None => vec![],
                },
            ]
            .concat(),
            _ => vec![],
        }
    }
}
impl Traversable for ast::AlterTableOperation {
    fn get_children(&self) -> Vec<Node> {
        match &self {
            ast::AlterTableOperation::AddConstraint(n) => {
                vec![Node::TableConstraint(n.to_owned())]
            }
            ast::AlterTableOperation::AddColumn { column_def } => {
                vec![Node::ColumnDef(column_def.to_owned())]
            }
            ast::AlterTableOperation::DropConstraint { name, .. } => {
                vec![Node::Ident(name.to_owned())]
            }
            ast::AlterTableOperation::DropColumn { column_name, .. } => {
                vec![Node::Ident(column_name.to_owned())]
            }
            ast::AlterTableOperation::RenamePartitions {
                old_partitions,
                new_partitions,
            } => [
                old_partitions
                    .iter()
                    .map(|n| Node::Expr(n.to_owned()))
                    .collect::<Vec<Node>>(),
                new_partitions
                    .iter()
                    .map(|n| Node::Expr(n.to_owned()))
                    .collect::<Vec<Node>>(),
            ]
            .concat(),
            ast::AlterTableOperation::AddPartitions { new_partitions, .. } => new_partitions
                .iter()
                .map(|n| Node::Expr(n.to_owned()))
                .collect::<Vec<Node>>(),
            ast::AlterTableOperation::DropPartitions { partitions, .. } => partitions
                .iter()
                .map(|n| Node::Expr(n.to_owned()))
                .collect::<Vec<Node>>(),
            ast::AlterTableOperation::RenameColumn {
                old_column_name,
                new_column_name,
            } => [
                value_child!(old_column_name, Ident),
                value_child!(new_column_name, Ident),
            ]
            .concat(),
            ast::AlterTableOperation::RenameTable { table_name } => {
                value_child!(table_name, ObjectName)
            }
            ast::AlterTableOperation::ChangeColumn {
                old_name,
                new_name,
                data_type,
                options,
            } => [
                value_child!(old_name, Ident),
                value_child!(new_name, Ident),
                value_child!(data_type, DataType),
                vec_child!(options, ColumnOption),
            ]
            .concat(),
            ast::AlterTableOperation::RenameConstraint { old_name, new_name } => {
                [value_child!(old_name, Ident), value_child!(new_name, Ident)].concat()
            }
            ast::AlterTableOperation::AlterColumn { column_name, op } => [
                value_child!(column_name, Ident),
                value_child!(op, AlterColumnOperation),
            ]
            .concat(),
        }
    }
}
impl Traversable for ast::BinaryOperator {
    fn get_children(&self) -> Vec<Node> {
        vec![]
    }
}
impl Traversable for ast::CloseCursor {
    fn get_children(&self) -> Vec<Node> {
        match &self {
            ast::CloseCursor::All => vec![],
            ast::CloseCursor::Specific { name } => value_child!(name, Ident),
        }
    }
}
impl Traversable for ast::ColumnOption {
    fn get_children(&self) -> Vec<Node> {
        match &self {
            ast::ColumnOption::Default(n) => value_child!(n, Expr),
            ast::ColumnOption::ForeignKey {
                foreign_table,
                referred_columns,
                on_delete,
                on_update,
            } => [
                value_child!(foreign_table, ObjectName),
                vec_child!(referred_columns, Ident),
                option_child!(on_delete, ReferentialAction),
                option_child!(on_update, ReferentialAction),
            ]
            .concat(),
            ast::ColumnOption::Check(n) => value_child!(n, Expr),
            ast::ColumnOption::CharacterSet(n) => value_child!(n, ObjectName),
            _ => vec![],
        }
    }
}
impl Traversable for ast::CommentObject {
    fn get_children(&self) -> Vec<Node> {
        vec![]
    }
}
impl Traversable for ast::CopyLegacyCsvOption {
    fn get_children(&self) -> Vec<Node> {
        match &self {
            ast::CopyLegacyCsvOption::ForceQuote(n) => vec_child!(n, Ident),
            ast::CopyLegacyCsvOption::ForceNotNull(n) => vec_child!(n, Ident),
            _ => vec![],
        }
    }
}
impl Traversable for ast::CopyLegacyOption {
    fn get_children(&self) -> Vec<Node> {
        match &self {
            ast::CopyLegacyOption::Csv(n) => vec_child!(n, CopyLegacyCsvOption),
            _ => vec![],
        }
    }
}
impl Traversable for ast::CopyOption {
    fn get_children(&self) -> Vec<Node> {
        match self {
            ast::CopyOption::Format(n) => value_child!(n, Ident),
            ast::CopyOption::ForceQuote(n) => vec_child!(n, Ident),
            ast::CopyOption::ForceNotNull(n) => vec_child!(n, Ident),
            ast::CopyOption::ForceNull(n) => vec_child!(n, Ident),
            _ => vec![],
        }
    }
}
impl Traversable for ast::CopyTarget {
    fn get_children(&self) -> Vec<Node> {
        vec![]
    }
}
impl Traversable for ast::CreateFunctionUsing {
    fn get_children(&self) -> Vec<Node> {
        vec![]
    }
}
impl Traversable for ast::DataType {
    fn get_children(&self) -> Vec<Node> {
        match &self {
            ast::DataType::Custom(n) => value_child!(n, ObjectName),
            ast::DataType::Array(n) => vec![Node::DataType(*n.to_owned())],
            _ => vec![],
        }
    }
}
impl Traversable for ast::DateTimeField {
    fn get_children(&self) -> Vec<Node> {
        vec![]
    }
}
impl Traversable for ast::DiscardObject {
    fn get_children(&self) -> Vec<Node> {
        vec![]
    }
}
impl Traversable for ast::Expr {
    fn get_children(&self) -> Vec<Node> {
        match self {
            ast::Expr::Identifier(n) => value_child!(n, Ident),
            ast::Expr::CompoundIdentifier(n) => vec_child!(n, Ident),
            ast::Expr::JsonAccess {
                left,
                operator,
                right,
            } => [
                box_child!(left, Expr),
                value_child!(operator, JsonOperator),
                box_child!(right, Expr),
            ]
            .concat(),
            ast::Expr::CompositeAccess { expr, key } => {
                [box_child!(expr, Expr), value_child!(key, Ident)].concat()
            }
            ast::Expr::IsFalse(n) => box_child!(n, Expr),
            ast::Expr::IsNotFalse(n) => box_child!(n, Expr),
            ast::Expr::IsTrue(n) => box_child!(n, Expr),
            ast::Expr::IsNotTrue(n) => box_child!(n, Expr),
            ast::Expr::IsNull(n) => box_child!(n, Expr),
            ast::Expr::IsNotNull(n) => box_child!(n, Expr),
            ast::Expr::IsUnknown(n) => box_child!(n, Expr),
            ast::Expr::IsNotUnknown(n) => box_child!(n, Expr),
            ast::Expr::IsDistinctFrom(n1, n2) => {
                [box_child!(n1, Expr), box_child!(n2, Expr)].concat()
            }
            ast::Expr::IsNotDistinctFrom(n1, n2) => {
                [box_child!(n1, Expr), box_child!(n2, Expr)].concat()
            }
            ast::Expr::InList { expr, list, .. } => {
                [box_child!(expr, Expr), vec_child!(list, Expr)].concat()
            }
            ast::Expr::InSubquery { expr, subquery, .. } => {
                [box_child!(expr, Expr), box_child!(subquery, Query)].concat()
            }
            ast::Expr::InUnnest {
                expr, array_expr, ..
            } => [box_child!(expr, Expr), box_child!(array_expr, Expr)].concat(),
            ast::Expr::Between {
                expr, low, high, ..
            } => [
                box_child!(expr, Expr),
                box_child!(low, Expr),
                box_child!(high, Expr),
            ]
            .concat(),
            ast::Expr::BinaryOp { left, op, right } => [
                box_child!(left, Expr),
                value_child!(op, BinaryOperator),
                box_child!(right, Expr),
            ]
            .concat(),
            ast::Expr::Like { expr, pattern, .. } => {
                [box_child!(expr, Expr), box_child!(pattern, Expr)].concat()
            }
            ast::Expr::ILike { expr, pattern, .. } => {
                [box_child!(expr, Expr), box_child!(pattern, Expr)].concat()
            }
            ast::Expr::SimilarTo { expr, pattern, .. } => {
                [box_child!(expr, Expr), box_child!(pattern, Expr)].concat()
            }
            ast::Expr::AnyOp(n) => box_child!(n, Expr),
            ast::Expr::AllOp(n) => box_child!(n, Expr),
            ast::Expr::UnaryOp { op, expr } => {
                [value_child!(op, UnaryOperator), box_child!(expr, Expr)].concat()
            }
            ast::Expr::Cast { expr, data_type } => {
                [box_child!(expr, Expr), value_child!(data_type, DataType)].concat()
            }
            ast::Expr::TryCast { expr, data_type } => {
                [box_child!(expr, Expr), value_child!(data_type, DataType)].concat()
            }
            ast::Expr::SafeCast { expr, data_type } => {
                [box_child!(expr, Expr), value_child!(data_type, DataType)].concat()
            }
            ast::Expr::AtTimeZone { timestamp, .. } => box_child!(timestamp, Expr),
            ast::Expr::Extract { field, expr } => {
                [value_child!(field, DateTimeField), box_child!(expr, Expr)].concat()
            }
            ast::Expr::Position { expr, r#in } => {
                [box_child!(expr, Expr), box_child!(r#in, Expr)].concat()
            }
            ast::Expr::Substring {
                expr,
                substring_from,
                substring_for,
            } => [
                box_child!(expr, Expr),
                option_box_child!(substring_from, Expr),
                option_box_child!(substring_for, Expr),
            ]
            .concat(),
            ast::Expr::Trim {
                expr,
                trim_where,
                trim_what,
            } => [
                box_child!(expr, Expr),
                option_child!(trim_where, TrimWhereField),
                option_box_child!(trim_what, Expr),
            ]
            .concat(),
            ast::Expr::Overlay {
                expr,
                overlay_what,
                overlay_from,
                overlay_for,
            } => [
                box_child!(expr, Expr),
                box_child!(overlay_what, Expr),
                box_child!(overlay_from, Expr),
                option_box_child!(overlay_for, Expr),
            ]
            .concat(),
            ast::Expr::Collate { expr, collation } => {
                [box_child!(expr, Expr), value_child!(collation, ObjectName)].concat()
            }
            ast::Expr::Nested(n) => box_child!(n, Expr),
            ast::Expr::Value(n) => vec![Node::Value(n.to_owned())],
            ast::Expr::TypedString { data_type, .. } => value_child!(data_type, DataType),
            ast::Expr::MapAccess { column, keys } => {
                [box_child!(column, Expr), vec_child!(keys, Expr)].concat()
            }
            ast::Expr::Function(n) => value_child!(n, Function),
            ast::Expr::Case {
                operand,
                conditions,
                results,
                else_result,
            } => [
                option_box_child!(operand, Expr),
                vec_child!(conditions, Expr),
                vec_child!(results, Expr),
                option_box_child!(else_result, Expr),
            ]
            .concat(),
            ast::Expr::Exists { subquery, .. } => box_child!(subquery, Query),
            ast::Expr::Subquery(n) => box_child!(n, Query),
            ast::Expr::ArraySubquery(n) => box_child!(n, Query),
            ast::Expr::ListAgg(n) => value_child!(n, ListAgg),
            ast::Expr::GroupingSets(n) => vec_vec_child!(n, Expr),
            ast::Expr::Cube(n) => vec_vec_child!(n, Expr),
            ast::Expr::Rollup(n) => vec_vec_child!(n, Expr),
            ast::Expr::Tuple(n) => vec_child!(n, Expr),
            ast::Expr::ArrayIndex { obj, indexes } => {
                [box_child!(obj, Expr), vec_child!(indexes, Expr)].concat()
            }
            ast::Expr::Array(n) => value_child!(n, Array),
        }
    }
}
impl Traversable for ast::FetchDirection {
    fn get_children(&self) -> Vec<Node> {
        match &self {
            ast::FetchDirection::Count { limit } => value_child!(limit, Value),
            ast::FetchDirection::Absolute { limit } => value_child!(limit, Value),
            ast::FetchDirection::Relative { limit } => value_child!(limit, Value),
            ast::FetchDirection::Forward { limit } => option_child!(limit, Value),
            ast::FetchDirection::Backward { limit } => option_child!(limit, Value),
            _ => vec![],
        }
    }
}
impl Traversable for ast::FileFormat {
    fn get_children(&self) -> Vec<Node> {
        vec![]
    }
}
impl Traversable for ast::FunctionArg {
    fn get_children(&self) -> Vec<Node> {
        match &self {
            ast::FunctionArg::Named { name, arg } => [
                value_child!(name, Ident),
                value_child!(arg, FunctionArgExpr),
            ]
            .concat(),
            ast::FunctionArg::Unnamed(n) => value_child!(n, FunctionArgExpr),
        }
    }
}
impl Traversable for ast::FunctionArgExpr {
    fn get_children(&self) -> Vec<Node> {
        match &self {
            ast::FunctionArgExpr::Expr(n) => value_child!(n, Expr),
            ast::FunctionArgExpr::QualifiedWildcard(n) => value_child!(n, ObjectName),
            _ => vec![],
        }
    }
}
impl Traversable for ast::GrantObjects {
    fn get_children(&self) -> Vec<Node> {
        match &self {
            ast::GrantObjects::AllSequencesInSchema { schemas: n } => vec_child!(n, ObjectName),
            ast::GrantObjects::AllTablesInSchema { schemas: n } => vec_child!(n, ObjectName),
            ast::GrantObjects::Schemas(n) => vec_child!(n, ObjectName),
            ast::GrantObjects::Sequences(n) => vec_child!(n, ObjectName),
            ast::GrantObjects::Tables(n) => vec_child!(n, ObjectName),
        }
    }
}
impl Traversable for ast::HiveDistributionStyle {
    fn get_children(&self) -> Vec<Node> {
        match &self {
            ast::HiveDistributionStyle::PARTITIONED { columns } => vec_child!(columns, ColumnDef),
            ast::HiveDistributionStyle::CLUSTERED {
                columns, sorted_by, ..
            } => [vec_child!(columns, Ident), vec_child!(sorted_by, ColumnDef)].concat(),
            ast::HiveDistributionStyle::SKEWED { columns, on, .. } => {
                [vec_child!(columns, ColumnDef), vec_child!(on, ColumnDef)].concat()
            }
            _ => vec![],
        }
    }
}
impl Traversable for ast::HiveIOFormat {
    fn get_children(&self) -> Vec<Node> {
        match &self {
            ast::HiveIOFormat::IOF {
                input_format,
                output_format,
            } => [
                value_child!(input_format, Expr),
                value_child!(output_format, Expr),
            ]
            .concat(),
            ast::HiveIOFormat::FileFormat { format } => value_child!(format, FileFormat),
        }
    }
}
impl Traversable for ast::HiveRowFormat {
    fn get_children(&self) -> Vec<Node> {
        vec![]
    }
}
impl Traversable for ast::JoinConstraint {
    fn get_children(&self) -> Vec<Node> {
        match &self {
            ast::JoinConstraint::On(n) => value_child!(n, Expr),
            ast::JoinConstraint::Using(n) => vec_child!(n, Ident),
            _ => vec![],
        }
    }
}
impl Traversable for ast::JoinOperator {
    fn get_children(&self) -> Vec<Node> {
        match &self {
            ast::JoinOperator::Inner(n) => value_child!(n, JoinConstraint),
            ast::JoinOperator::LeftOuter(n) => value_child!(n, JoinConstraint),
            ast::JoinOperator::RightOuter(n) => value_child!(n, JoinConstraint),
            ast::JoinOperator::FullOuter(n) => value_child!(n, JoinConstraint),
            _ => vec![],
        }
    }
}
impl Traversable for ast::JsonOperator {
    fn get_children(&self) -> Vec<Node> {
        vec![]
    }
}
impl Traversable for ast::KillType {
    fn get_children(&self) -> Vec<Node> {
        vec![]
    }
}
impl Traversable for ast::ListAggOnOverflow {
    fn get_children(&self) -> Vec<Node> {
        match &self {
            ast::ListAggOnOverflow::Truncate { filler, .. } => option_box_child!(filler, Expr),
            _ => vec![],
        }
    }
}
impl Traversable for ast::LockType {
    fn get_children(&self) -> Vec<Node> {
        vec![]
    }
}
impl Traversable for ast::MergeClause {
    fn get_children(&self) -> Vec<Node> {
        match &self {
            ast::MergeClause::MatchedUpdate {
                predicate,
                assignments,
            } => [
                option_child!(predicate, Expr),
                vec_child!(assignments, Assignment),
            ]
            .concat(),
            ast::MergeClause::MatchedDelete(n) => option_child!(n, Expr),
            ast::MergeClause::NotMatched {
                predicate,
                columns,
                values,
            } => [
                option_child!(predicate, Expr),
                vec_child!(columns, Ident),
                value_child!(values, Values),
            ]
            .concat(),
        }
    }
}
impl Traversable for ast::ObjectType {
    fn get_children(&self) -> Vec<Node> {
        vec![]
    }
}
impl Traversable for ast::OffsetRows {
    fn get_children(&self) -> Vec<Node> {
        vec![]
    }
}
impl Traversable for ast::OnCommit {
    fn get_children(&self) -> Vec<Node> {
        vec![]
    }
}
impl Traversable for ast::OnInsert {
    fn get_children(&self) -> Vec<Node> {
        match &self {
            ast::OnInsert::DuplicateKeyUpdate(n) => vec_child!(n, Assignment),
            _ => vec![],
        }
    }
}
impl Traversable for ast::Privileges {
    fn get_children(&self) -> Vec<Node> {
        match &self {
            ast::Privileges::Actions(n) => vec_child!(n, Action),
            _ => vec![],
        }
    }
}
impl Traversable for ast::ReferentialAction {
    fn get_children(&self) -> Vec<Node> {
        vec![]
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
        }
    }
}
impl Traversable for ast::SetOperator {
    fn get_children(&self) -> Vec<Node> {
        vec![]
    }
}
impl Traversable for ast::ShowCreateObject {
    fn get_children(&self) -> Vec<Node> {
        vec![]
    }
}
impl Traversable for ast::ShowStatementFilter {
    fn get_children(&self) -> Vec<Node> {
        match &self {
            ast::ShowStatementFilter::Where(n) => value_child!(n, Expr),
            _ => vec![],
        }
    }
}
impl Traversable for ast::SqliteOnConflict {
    fn get_children(&self) -> Vec<Node> {
        vec![]
    }
}
impl Traversable for ast::Statement {
    fn get_children(&self) -> Vec<Node> {
        match self {
            ast::Statement::Query(q) => {
                vec![Node::Query(*q.to_owned())]
            }
            ast::Statement::Analyze {
                table_name,
                partitions,
                columns,
                ..
            } => [
                value_child!(table_name, ObjectName),
                option_vec_child!(partitions, Expr),
                vec_child!(columns, Ident),
            ]
            .concat(),
            ast::Statement::Truncate {
                table_name,
                partitions,
            } => [
                value_child!(table_name, ObjectName),
                option_vec_child!(partitions, Expr),
            ]
            .concat(),
            ast::Statement::Msck {
                table_name,
                partition_action,
                ..
            } => [
                value_child!(table_name, ObjectName),
                option_child!(partition_action, AddDropSync),
            ]
            .concat(),
            ast::Statement::Insert {
                or,
                table_name,
                columns,
                source,
                partitioned,
                after_columns,
                on,
                ..
            } => [
                option_child!(or, SqliteOnConflict),
                value_child!(table_name, ObjectName),
                vec_child!(columns, Ident),
                box_child!(source, Query),
                option_vec_child!(partitioned, Expr),
                vec_child!(after_columns, Ident),
                option_child!(on, OnInsert),
            ]
            .concat(),
            ast::Statement::Directory {
                file_format,
                source,
                ..
            } => [
                option_child!(file_format, FileFormat),
                box_child!(source, Query),
            ]
            .concat(),
            ast::Statement::Copy {
                table_name,
                columns,
                target,
                options,
                legacy_options,
                ..
            } => [
                value_child!(table_name, ObjectName),
                vec_child!(columns, Ident),
                value_child!(target, CopyTarget),
                vec_child!(options, CopyOption),
                vec_child!(legacy_options, CopyLegacyOption),
            ]
            .concat(),
            ast::Statement::Close { cursor } => value_child!(cursor, CloseCursor),
            ast::Statement::Update {
                table,
                assignments,
                from,
                selection,
            } => [
                value_child!(table, TableWithJoins),
                vec_child!(assignments, Assignment),
                option_child!(from, TableWithJoins),
                option_child!(selection, Expr),
            ]
            .concat(),
            ast::Statement::Delete {
                table_name,
                using,
                selection,
            } => [
                value_child!(table_name, TableFactor),
                option_child!(using, TableFactor),
                option_child!(selection, Expr),
            ]
            .concat(),
            ast::Statement::CreateView {
                name,
                columns,
                query,
                with_options,
                ..
            } => [
                value_child!(name, ObjectName),
                vec_child!(columns, Ident),
                box_child!(query, Query),
                vec_child!(with_options, SqlOption),
            ]
            .concat(),
            ast::Statement::CreateTable {
                name,
                columns,
                constraints,
                hive_distribution,
                hive_formats,
                table_properties,
                with_options,
                file_format,
                query,
                like,
                clone,
                on_commit,
                ..
            } => [
                value_child!(name, ObjectName),
                vec_child!(columns, ColumnDef),
                vec_child!(constraints, TableConstraint),
                value_child!(hive_distribution, HiveDistributionStyle),
                option_child!(hive_formats, HiveFormat),
                vec_child!(table_properties, SqlOption),
                vec_child!(with_options, SqlOption),
                option_child!(file_format, FileFormat),
                option_box_child!(query, Query),
                option_child!(like, ObjectName),
                option_child!(clone, ObjectName),
                option_child!(on_commit, OnCommit),
            ]
            .concat(),
            ast::Statement::CreateVirtualTable {
                name,
                module_name,
                module_args,
                ..
            } => [
                value_child!(name, ObjectName),
                value_child!(module_name, Ident),
                vec_child!(module_args, Ident),
            ]
            .concat(),
            ast::Statement::CreateIndex {
                name,
                table_name,
                columns,
                ..
            } => [
                value_child!(name, ObjectName),
                value_child!(table_name, ObjectName),
                vec_child!(columns, OrderByExpr),
            ]
            .concat(),
            ast::Statement::AlterTable { name, operation } => [
                value_child!(name, ObjectName),
                value_child!(operation, AlterTableOperation),
            ]
            .concat(),
            ast::Statement::Drop {
                object_type, names, ..
            } => [
                value_child!(object_type, ObjectType),
                vec_child!(names, ObjectName),
            ]
            .concat(),
            ast::Statement::Declare { name, query, .. } => {
                [value_child!(name, Ident), box_child!(query, Query)].concat()
            }
            ast::Statement::Fetch {
                name,
                direction,
                into,
            } => [
                value_child!(name, Ident),
                value_child!(direction, FetchDirection),
                option_child!(into, ObjectName),
            ]
            .concat(),
            ast::Statement::Discard { object_type } => value_child!(object_type, DiscardObject),
            ast::Statement::SetRole { role_name, .. } => option_child!(role_name, Ident),
            ast::Statement::SetVariable {
                variable, value, ..
            } => [value_child!(variable, ObjectName), vec_child!(value, Expr)].concat(),
            ast::Statement::ShowVariable { variable } => vec_child!(variable, Ident),
            ast::Statement::ShowVariables { filter } => option_child!(filter, ShowStatementFilter),
            ast::Statement::ShowCreate { obj_type, obj_name } => [
                value_child!(obj_type, ShowCreateObject),
                value_child!(obj_name, ObjectName),
            ]
            .concat(),
            ast::Statement::ShowColumns {
                table_name, filter, ..
            } => [
                value_child!(table_name, ObjectName),
                option_child!(filter, ShowStatementFilter),
            ]
            .concat(),
            ast::Statement::ShowTables {
                db_name, filter, ..
            } => [
                option_child!(db_name, Ident),
                option_child!(filter, ShowStatementFilter),
            ]
            .concat(),
            ast::Statement::ShowCollation { filter } => option_child!(filter, ShowStatementFilter),
            ast::Statement::Use { db_name } => value_child!(db_name, Ident),
            ast::Statement::StartTransaction { modes } => vec_child!(modes, TransactionMode),
            ast::Statement::SetTransaction {
                modes, snapshot, ..
            } => [
                vec_child!(modes, TransactionMode),
                option_child!(snapshot, Value),
            ]
            .concat(),
            ast::Statement::Comment {
                object_type,
                object_name,
                ..
            } => [
                value_child!(object_type, CommentObject),
                value_child!(object_name, ObjectName),
            ]
            .concat(),
            ast::Statement::Commit { .. } => vec![],
            ast::Statement::Rollback { .. } => vec![],
            ast::Statement::CreateSchema { schema_name, .. } => {
                value_child!(schema_name, ObjectName)
            }
            ast::Statement::CreateDatabase { db_name, .. } => value_child!(db_name, ObjectName),
            ast::Statement::CreateFunction { name, using, .. } => [
                value_child!(name, ObjectName),
                option_child!(using, CreateFunctionUsing),
            ]
            .concat(),
            ast::Statement::Assert { condition, message } => {
                [value_child!(condition, Expr), option_child!(message, Expr)].concat()
            }
            ast::Statement::Grant {
                privileges,
                objects,
                grantees,

                granted_by,
                ..
            } => [
                value_child!(privileges, Privileges),
                value_child!(objects, GrantObjects),
                vec_child!(grantees, Ident),
                option_child!(granted_by, Ident),
            ]
            .concat(),
            ast::Statement::Revoke {
                privileges,
                objects,
                grantees,
                granted_by,
                ..
            } => [
                value_child!(privileges, Privileges),
                value_child!(objects, GrantObjects),
                vec_child!(grantees, Ident),
                option_child!(granted_by, Ident),
            ]
            .concat(),
            ast::Statement::Deallocate { name, .. } => value_child!(name, Ident),
            ast::Statement::Execute { name, parameters } => {
                [value_child!(name, Ident), vec_child!(parameters, Expr)].concat()
            }
            ast::Statement::Prepare {
                name,
                data_types,
                statement,
            } => [
                value_child!(name, Ident),
                vec_child!(data_types, DataType),
                box_child!(statement, Statement),
            ]
            .concat(),
            ast::Statement::Kill { modifier, .. } => option_child!(modifier, KillType),
            ast::Statement::ExplainTable { table_name, .. } => value_child!(table_name, ObjectName),
            ast::Statement::Explain { statement, .. } => box_child!(statement, Statement),
            ast::Statement::Savepoint { name } => value_child!(name, Ident),
            ast::Statement::Merge {
                table,
                source,
                on,
                clauses,
                ..
            } => [
                value_child!(table, TableFactor),
                value_child!(source, TableFactor),
                box_child!(on, Expr),
                vec_child!(clauses, MergeClause),
            ]
            .concat(),
            _ => vec![],
        }
    }
}
impl Traversable for ast::TableConstraint {
    fn get_children(&self) -> Vec<Node> {
        match &self {
            ast::TableConstraint::Unique { name, columns, .. } => {
                [option_child!(name, Ident), vec_child!(columns, Ident)].concat()
            }
            ast::TableConstraint::ForeignKey {
                name,
                columns,
                foreign_table,
                referred_columns,
                on_delete,
                on_update,
            } => [
                option_child!(name, Ident),
                vec_child!(columns, Ident),
                value_child!(foreign_table, ObjectName),
                vec_child!(referred_columns, Ident),
                option_child!(on_delete, ReferentialAction),
                option_child!(on_update, ReferentialAction),
            ]
            .concat(),
            ast::TableConstraint::Check { name, expr } => {
                [option_child!(name, Ident), box_child!(expr, Expr)].concat()
            }
        }
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
                    children.extend(n.iter().map(|n| Node::FunctionArg(n.to_owned())));
                };
                children.extend(with_hints.iter().map(|n| Node::Expr(n.to_owned())));

                children
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
                children
            }
            ast::TableFactor::TableFunction { expr, alias } => {
                let mut children = vec![Node::Expr(expr.to_owned())];
                if let Some(n) = alias {
                    children.push(Node::TableAlias(n.to_owned()));
                };

                children
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
                children
            }
            ast::TableFactor::NestedJoin {
                table_with_joins,
                alias,
            } => {
                let mut children = vec![Node::TableWithJoins(*table_with_joins.to_owned())];
                if let Some(n) = alias {
                    children.push(Node::TableAlias(n.to_owned()));
                };
                children
            }
        }
    }
}
impl Traversable for ast::TransactionAccessMode {
    fn get_children(&self) -> Vec<Node> {
        vec![]
    }
}
impl Traversable for ast::TransactionIsolationLevel {
    fn get_children(&self) -> Vec<Node> {
        vec![]
    }
}
impl Traversable for ast::TransactionMode {
    fn get_children(&self) -> Vec<Node> {
        match &self {
            ast::TransactionMode::AccessMode(n) => value_child!(n, TransactionAccessMode),
            ast::TransactionMode::IsolationLevel(n) => value_child!(n, TransactionIsolationLevel),
        }
    }
}
impl Traversable for ast::TrimWhereField {
    fn get_children(&self) -> Vec<Node> {
        vec![]
    }
}
impl Traversable for ast::UnaryOperator {
    fn get_children(&self) -> Vec<Node> {
        vec![]
    }
}
impl Traversable for ast::Value {
    fn get_children(&self) -> Vec<Node> {
        match self {
            ast::Value::Interval {
                value,
                leading_field,
                last_field,
                ..
            } => [
                box_child!(value, Expr),
                option_child!(leading_field, DateTimeField),
                option_child!(last_field, DateTimeField),
            ]
            .concat(),
            _ => vec![],
        }
    }
}
impl Traversable for ast::WindowFrameBound {
    fn get_children(&self) -> Vec<Node> {
        vec![]
    }
}
impl Traversable for ast::WindowFrameUnits {
    fn get_children(&self) -> Vec<Node> {
        vec![]
    }
}
