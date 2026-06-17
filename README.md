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

#Updates
**Update 1:**
Version 1 is complete. The lexer turns characters into tokens, and the parser recognizes 'int main() { return n; }'. The AST stores the return statement, and code generation places the result in the system V AMD64 return value register. All the code is in the src file.
