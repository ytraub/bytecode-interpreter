use crate::scanner::{Scanner, TokenType};

pub fn compile(source: String) -> Result<(), String> {
    let mut scanner = Scanner::new(source);

    let mut line = -1;
    loop {
        let token = scanner.scan_token();
        let new_line = token.get_line();
        if new_line != line {
            print!("{:04} ", new_line);
            line = new_line;
        } else {
            print!("   | ");
        }

        println!("{:02?} '{}'", token.get_type(), token.get_lexeme());

        if token.get_type() == &TokenType::EOF {
            break;
        };
    }

    return Ok(());
}
