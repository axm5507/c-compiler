# c-compiler
In this project, I made a c compiler implemented in idiomatic rust. It contains a handwritten recursive-descent parser and a basic pipeline.

Lexer -> Parser -> AST -> Semantic Analysis -> Code Generation -> x86-64 Assembly

Right now, the following 7 versions have been implemented:

1. Bare-bones, just int main() { return 1; }
2. Arithmetic expressions and PEMDAS
3. Local variables, symbol tables
4. if and else statements, for and while loops, comparisons, short circuit logic
5. Functions, parameters, calls, system V AMD64 integer registers
6. Pointers, address of, dereferencing
7. Arrays, indexing, structs, alignment, layout

**Update**
All 7 versions are now complete. Current limitations to the compiler that I may work on sometime in the future include the lack of global variables, string literals, no casts, and no sizeof operator. Some other things to add in the future are adding a three address code layer between semantic analysis and codegen to make handling later optimization passes easier, register allocation without spilling, and constant folding.

# How to Run

**Prerequisites:** Rust toolchain (`rustup`/`cargo`). To assemble and execute the output, a C compiler like `gcc` or `clang` that can read `.s` files is required.

**Compile a `.c` file to assembly:**
```sh
cargo run -- examples/v7_arrays_structs.c
```
This prints x86-64 Intel-syntax assembly to stdout.

**Assemble and run the result (Linux/macOS):**
```sh
cargo run -- examples/v7_arrays_structs.c > out.s
gcc -o out out.s
./out
echo $? 
```

**Assemble and run (one-liner):**
```sh
cargo run -- examples/v7_arrays_structs.c | gcc -x assembler - -o out && ./out; echo $?
```

**Run the example files:**
```
examples/v4_control_flow.c 
examples/v5_functions.c 
examples/v6_pointers.c     
examples/v7_arrays_structs.c 
```

**Build without running:**
```sh
cargo build --release        
```

**Clean build artifacts:**
```sh
cargo clean
```

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

**Update 6:**
Version 6 is now complete. int* pointer types have been added along with two unary operators: &x(address of) and *x(dereferencing). The type system now tracks pointer depth(ie int\*, int**, etc) and enforces that & only applies to lvalues and * only applies to pointers. Assignmetn is generalized from simple variable writes to lvalue aware stores so *x = 5 and **xx = 5 both work fine. Codegen uses a two function approach where gen_expr produces a value in rax and gen_addr produces an address in rax. Assignment composes the two.

**Update 7:**
Version 7 is finished. Fixed size arrays and structs have been added. The theory is that arrays are contiguous repeated elements with the 0th index at the lowest address, and structs pack fields at sequentially increasing offsets with each field aligned to 8 bytes. The total struct size is rounded up to an 8 byte boundary because all the types in the compiler(int, ptr, arr, struct) are either 8 bytes or a multiple of 8 bytes. In the lexer, I introduced 5 new tokens, LBracket, RBracket, Dot, Arrow, and StructKw. In AST, the program now carries a structs: Vec<StructDecl> list alongside functions. Expr gained Index(Box<Expr>, Box<Expr>) for x[i] and Field(Box<Expr>, String) for p.x. The -> operator is desugared in the parser into Field(Unary(Deref, ...), field) so the rest of the compiler only has to understand dot access. In the parser, parse_program now dispatches on the leading token. struct starts a top level parse_struct_decl, and int starts a function. parse_postfix level was added between parse_unary and parse_primary and loops consuming [], .ident, and ->ident. Postfix binds tighter than any prefix unary operator. parse_decl checks for [N] after the var name and wraps base type in Array. parse_type handles struct Name with optional trailing stars. For semantic analysis, a new pre pass processes program.structs in declaration order and computes a StructLayout for each struct and records total size. The current single variable allocation in declare was generalized so arrays and structs can consume as many bytes as they actually need. SymbolTable gains a types: HashMap<String, Type> field so codegen can tell at codegen timme whether a var is scalar, array, or struct. is_lvalue is now extended to include Index and Field. check_expr now handles Index and Field. In Codegen, a new type_size helper and expr_type function let codegen now compute element sizes and field offsets without re running sema. gen_expr(Var(name)) is type aware. gen_addr gained two new arms, index and field. Stmt::Decl for aggregate types emits slot by slot zeroing with xor rax, rax followed by one move [rbp-N], rax per 8 byte word. 
