fn tokenizer(filename: String) -> Result<Vec<Token>> {
    let mut tokens: Vec<Token> = Vec::new();

    let file = File::open(filename)?;
    let mut buffer = BufReader::new(file);

    let mut line = Vec::new();
    let mut line_num = 0;
    loop {
        let bytes_read = buffer.read_until(b' ', &mut line)?;
        if bytes_read == 0 {
            break;
        } else if bytes_read == 1 {
            continue;
        } else if bytes_read == 2 {
            match line[0] {
                b'*' => {
                    tokens.push(Token::Asterisk(TokenInfo {
                        lexeme: "*".to_string(),
                        literal: "*".to_string(),
                        line: line_num,
                    }));
                }
                b'\n' => {
                    tokens.push(Token::NewLine(TokenInfo {
                        lexeme: "\n".to_string(),
                        literal: "\n".to_string(),
                        line: line_num,
                    }));
                    line_num += 1;
                }
                _ => {}
            }
        } else {
            let mut lexeme = String::from_utf8(line.clone())?;
            if lexeme[0] == "\"" {
                let mut literal = String::new();
                let mut i = 1;
            }
            lexeme.pop(); // remove the space
            match lexeme.as_str() {
                "repeat" => {
                    tokens.push(Token::Repeat(TokenInfo {
                        lexeme: lexeme.clone(),
                        literal: lexeme.clone(),
                        line: line_num,
                    }));
                }
                "calendar" => {
                    tokens.push(Token::Calendar(TokenInfo {
                        lexeme: lexeme.clone(),
                        literal: lexeme.clone(),
                        line: line_num,
                    }));
                }
                _ => {
                    tokens.push(Token::Identifier(TokenInfo {
                        lexeme: lexeme.clone(),
                        literal: lexeme.clone(),
                        line: line_num,
                    }));
                }
            }
        }
    }

    Ok(tokens)
}
