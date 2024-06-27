# AST
Check for the following invariants
1. Name must not be equal to any value in RESERVED_KEYWORDS
2. Statement::MultipleAssignment must have an equal number of vars and expressions
3. BinaryOperator::Concat -> Both ExprInner of whatever is calling the binary operation
   must resolve to either a String or something that implements ToString
