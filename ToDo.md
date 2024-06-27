## AST
Check for the following invariants
1. Name must not be equal to any value in RESERVED_KEYWORDS
2. Statement::MultipleAssignment must have an equal number of vars and expressions
3. BinaryOperator::Concat -> Both ExprInner of whatever is calling the binary operation
   must resolve to either a String or something that implements ToString

## Frontend
Variables need to be scoped, need to account for local vs global

## Presentation
- show minimal program
  - two functions. one adds two numbers and returns it,
    the other takes an input, does a string concat, and
    then prints it. bonus points for an if statement
- tokenization
  - explain tokenization and show a screenshot of the
    tokens of the minimal program. explain one token
    in detail, ex its span and inner
- AST
  - explain what an AST is and show a screenshot of the
    AST of the minimal program. Explain its structure
- Frontend
  - Variable storage
  - Registering std lib functions
    - dyn functions
  - Walking AST
- Run program to show it actually works?
