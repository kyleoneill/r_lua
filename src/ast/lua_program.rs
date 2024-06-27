#![allow(dead_code)]
// TODO: Remove this once everything is being used

#[derive(Debug)]
pub struct LuaProgram {
    pub block: Block,
}

#[derive(Debug)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub return_statement: Option<ReturnStatement>,
}

#[derive(Debug)]
pub enum Statement {
    Empty,
    MultipleAssignment(VarList, ExpressionList),
    FunctionCall(Var, Args),
    Label(String),
    Break,
    GoTo(String),
    DoBlockEnd(Block),
    WhileExprDoBlockEnd(Expression, Block),
    RepeatBlockUntilExpr(Block, Expression),
    IfBlock((Expression, Block), Vec<(Expression, Block)>, Option<Block>),
    ForEach(String, Expression, Expression, Option<Expression>, Block),
    ForList(NameList, ExpressionList, Block),
    Function(FunctionName, FunctionBody),
    LocalFunction(String, FunctionBody),
    // TODO: local AttributeNameList = ExpressionList
}

#[derive(Debug)]
pub struct ReturnStatement {
    pub expression_list: Option<ExpressionList>,
}

#[derive(Debug)]
pub struct ExpressionList {
    pub expressions: Vec<Expression>,
}

#[derive(Debug)]
pub struct NameList {
    pub names: Vec<String>,
}

#[derive(Debug)]
pub enum Expression {
    Expr(Expr),
    Binary(BinaryOperator, Expr, Expr),
}

#[derive(Debug)]
pub enum Expr {
    Nil,
    Boolean(bool),
    Numerical(NumberKind),
    LiteralString(String),
    Expansion(Expansion),
    FunctionDef(FunctionBody),
    Prefix(Box<PrefixExpression>),
    Unary(UnaryOperator, Box<Expression>)
    // TODO: TableConstructor,
}

#[derive(Debug)]
pub enum PrefixExpression {
    Var(Var),
    FunctionCall(Var, Args),
    Expression(Expression)
}

#[derive(Debug)]
pub enum BinaryOperator {
    MathOperator(MathOperator),
    BitwiseOperator(BitwiseOperator),
    Concat,
    BooleanOperator(BooleanOperator),
}

#[derive(Debug)]
pub enum MathOperator {
    Plus,
    Minus,
    Multiply,
    FloatDivision,
    FloorDivision,
    Exponent,
    Mod,
}

#[derive(Debug)]
pub enum BitwiseOperator {
    And,
    Or,
    ExclusiveOr,
    RightShift,
    LeftShift,
}

#[derive(Debug)]
pub enum BooleanOperator {
    LessThan,
    LessThanEqualTo,
    GreaterThan,
    GreaterThanEqualTo,
    Equal,
    Unequal, // ~=
    And,
    Or,
}

#[derive(Debug)]
pub enum UnaryOperator {
    UnaryMinus,
    Not,
    Length,          // #
    BitwiseUnaryNot, // ~
}

#[derive(Debug)]
pub struct VarList {
    pub vars: Vec<Var>,
}

#[derive(Debug)]
pub enum Var {
    NestedAccess(Vec<String>),
    #[allow(clippy::enum_variant_names)]
    VarName(String),
    TableAccess(String, Expression),
}

#[derive(Debug)]
pub struct FunctionName {
    pub outer_name: String,
    pub accessors: Vec<String>,
    pub pass_self: Option<String>, // foo.bar.baz:thing
                                   // This results in { outer_name: "foo", accessor: vec!["bar", "baz"], pass_self: Some("thing") }
                                   // In Lua, calling a function with `:`, like `x:bar(3, 4)` passes self and resolves to
                                   // x.bar(x, 3, 4)
}

#[derive(Debug)]
pub enum Parameters {
    Normal(NameList, Option<Expansion>),
    Expanded(Expansion),
}

#[derive(Debug)]
pub struct FunctionBody {
    pub parameters: Option<Parameters>,
    pub block: Block,
}

#[derive(Debug)]
pub enum Args {
    ExpressionList(Option<ExpressionList>),
    // TODO: Support for TableConstructor and LiteralString variants
    // I think these are only useful if the recursion problem is fixed and PrefixExpression
    // is added back to the grammar
}

#[derive(Debug)]
pub enum NumberKind {
    Int(i64),
    Float(f64),
}

#[derive(Debug)]
pub struct Expansion;
