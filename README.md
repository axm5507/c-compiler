# c-compiler
In this project, I will be making a c compiler implemented in idiomatic rust. It will contain a handwritten recursive-descent parser and a basic pipeline.

Lexer -> Parser -> AST -> Semantic Analysis -> Code Generation -> x86-64 Assembly

Right now, I plan on implementing the following 7 versions:

1. Bare-bones, just int main() { return 1; }
2. Arithmetic expressions and PEMDAS
3. Local variables, symbol tables
4. if and else statements, for and while loops, comparisons, short circuit logic
5. Functions, parameters, calls, system V AMD64 integer registers
6. Pointers, address of, dereferencing
7. Arrays, indexing, structs, alignment, layout

# A few questions I thought of while starting:

**Why this project?**
I've been becoming more and more interested in computer architecture lately, and making a simple compiler seemed like a good idea to help understand how everything works under the hood. It will force me to learn things like instruction sets, registers, and memory layout, and is directly connected to CPU architecture.

**Why Rust?**
I found rust's enum and pattern matching to be much cleaner than C code, where I'd have to use unions and manual type tags. It also has a strict memory model that makes sure a bunch of different types of bugs will be blocked at compile time. There are some tradeoffs, like less low level control and pretty complex data structures, but I also wanted to grow my knowledge in rust.

# Updates

**Update 1:**
Version 1 is complete. The lexer turns characters into tokens, and the parser recognizes 'int main() { return n; }'. The AST stores the return statement, and code generation places the result in the system V AMD64 return value register. All the code is in the src file.

**Update 2:**
Version 2 is complete. While version 1 only had integer literals, version 2 adds unary and binary expressions. I used recursive descent, using one function per precendence level, to implement associativity and PEMDAS. For AST, I added 'Expr::Binary', 'Expr::Unary', 'BinaryOp', and 'UnaryOp'. Parser changes include 'parse_additive' loops for addition and subtraction operations, 'parse_multiplicative' loops for multiplication, division, and modulus operations, and 'parse_unary' for negativity. In this version, arithmetic operators require integer operands. In codegen, the changes include generating left and right operands, using stack temporaries, and emitting 'add', 'sub', 'imul', 'idiv', or remainder from 'rdx'.

**Update 3:**
Version 3 is now complete and variables are supported. For AST, the Function.body is now Vec<Stmt> so a function can hold multiple statements instead of one return. I also added Stmt::Decl(VarDecl) and Stmt::Expr(Expr) for assignemnts, the VarDecl { name, init: Option<Expr> } struct, and Expr::Var(String) and Expr::Assign(String, Box<Expr>). For lexer, I added the = token. For the parser, I made parse_program loop until a closed bracket appears. The parse_stmt dispaches to parse_decl, return, or an expression statement. parse_assign is at the bottom of the precedence ladder and is also right associative(as it should be), while rejecting non variable assignment targets. I added the file for semantic analysis which walks the AST with a stack of scoped hash maps, rejecting undeclared variables and dublicate declarations in the same scope. It also checks initalizers before the name enters scope so int x = x is an error. Finally, it assigns each local an 8 byte slot and returns a SymbolTable { offsets, stack_size } with size rounded to 16 bytes for ABI alignment. For codegen, it now stores initializers into [rbp-offset], reads variables via lea/mov, and stores assignment results while keeping the value in rax for chaining. 

**Update 4:**
Version 4 has now been added. I implemented a control flow that lowers to labels and conditional jumps. In the Lexer, I added new keywords for if, else, while, and for, comparison operators ==, !=, <, <=, >, and >=, logical operators &&, ||, and // for comments. Two character operators like == are matched with a one char lookahead. In AST, I added new statements Block, If, While, and For, comparison variants on BinaryOp, and a seperate Expr::Logical node for && and ||. In the parser, I added dedicated parsers for each statement form, and a precedence ladder that is in the comments there. Each 'for' header clause is optional. For semantics, conditions are validated as ordinary expressions and the checker recurses into all branch/loop bodies. Comparisons yield int. For Codegen, I added a threaded label counter that mints unique '.L<n>' targets. if, while, and for emit cmp rax, 0 + je/jmp with loop back edges, comparisons use cmp + setcc + movzx, and && and || emit branches that skip right operand.

**Update 5:**
Version 5 has now been finished. The compiler can now handle multiple functions with parameters, and calls following the System V AMD64 calling convention. In the lexer, the comma token was implemented for argument/parameter lists. In AST, Program is now  a list of functions, Function carries ordered parameter names, and Expr::Call(name, args) was added to call functions. The parser now parses multiple int name(params){} definitions, comma separated parameter lists, and name(args) call expressions. In semantics, there is now a two pass design. In the first pass, every function signature is collected so calls in the second pass can target functions defined later in the file(also for recursion to work) and check each body/verify if calls are valid. Parameters are laid out as the first local slots, and functions are capped at 6 parameters(only cpu registers). In codegen, arguments arrive in rdi, rsi, rdx, rcx, r8, and r9 and are spilled into stack slots in the prologue. Return values come back in rax. A call evaluates and pushes its arguments, pops them into the right registers, keeps rsp 16 byte aligned through a push pop depth counter, and emits call name. Each function gets its own return label.