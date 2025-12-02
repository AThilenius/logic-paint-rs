use peg::parser;

parser! {
    pub grammar riscv_asm() for str {
        // Main rule - parses a whole assembly file
        pub rule program() -> Vec<Line>
            = l:line() ** ("\n" / comment()) _ "\n"* { l }

        // A single line of assembly (could be instruction, label, or empty)
        rule line() -> Line
            = _ l:(instruction() / label() / directive() / empty()) _ comment()? { l }

        // Assembly instruction
        rule instruction() -> Line
            = opcode:opcode() _ operands:operands() {
                Line::Instruction(opcode, operands)
            }

        // Label definition
        rule label() -> Line
            = name:label_name() ":" { Line::Label(name) }

        // Assembler directive
        rule directive() -> Line
            = "." name:ident() {
                Line::Directive(name)
            }

        // Empty line
        rule empty() -> Line
            = { Line::Empty }

        // Operands for an instruction
        rule operands() -> Vec<Operand>
            = operand() ** ("," _)

        // Single operand
        rule operand() -> Operand
            = register() / immediate() / label_ref()

        // Register operand
        rule register() -> Operand
            = reg:register_name() { Operand::Register(reg) }

        // Immediate operand (numeric)
        rule immediate() -> Operand
            = imm:number() { Operand::Immediate(imm) }

        // Label reference
        rule label_ref() -> Operand
            = label:label_name() { Operand::Label(label) }

        // RISC-V opcodes
        rule opcode() -> String
            = op:$(
                "add"/ "sub"
            ) { op.to_string() }

        // Register names (x0-x31 or ABI names)
        rule register_name() -> String
            = reg:$("r0" / "r1" / "r2" / "r3" / "r4" / "r5" / "r6" / "r7") { reg.to_string() }

        // Label name
        rule label_name() -> String
            = name:$(['a'..='z' | 'A'..='Z' | '_']['a'..='z' | 'A'..='Z' | '0'..='9' | '_']*) {
                name.to_string()
            }

        // Identifier (for directives)
        rule ident() -> String
            = id:$(['a'..='z' | 'A'..='Z' | '_']['a'..='z' | 'A'..='Z' | '0'..='9' | '_']*) {
                id.to_string()
            }

        // Numeric literal (decimal or hex)
        rule number() -> i32
            = n:$("-"? ("0x" ['0'..='9' | 'a'..='f' | 'A'..='F']+ / ['0'..='9']+)) {
                if n.starts_with("0x") {
                    i32::from_str_radix(&n[2..], 16).unwrap()
                } else {
                    n.parse().unwrap()
                }
            }

        // Quoted string
        rule quoted_string() -> String
            = "\"" s:$([^ '"']*) "\"" { s.to_string() }

        // Comment
        rule comment() -> ()
            = _ "#" [^'\n']* { () }

        // Whitespace
        rule _() -> ()
            = [' ' | '\t']* { () }
    }
}

// AST types
#[derive(Debug, PartialEq)]
pub enum Line {
    Instruction(String, Vec<Operand>),
    Label(String),
    Directive(String),
    Empty,
}

#[derive(Debug, PartialEq)]
pub enum Operand {
    Register(String),
    Immediate(i32),
    Label(String),
    Offset(i32, String),
}

// Example usage
pub fn parser_test() {
    let asm_code = r#"
        add x1, x0, 42      # Load immediate
        sub x2, 0x10000
    "#;

    match riscv_asm::program(asm_code) {
        Ok(parsed) => println!("{:#?}", parsed),
        Err(e) => println!("Parse error: {}", e),
    }
}
