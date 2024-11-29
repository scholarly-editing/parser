use crate::core::state::Token;

pub struct Document<'a> {
    pub pages: Vec<Vec<Token<'a>>>,
}

impl<'a> Document<'a> {
    pub fn new(tokens: Vec<Token<'a>>) -> Self {
        let mut pages: Vec<Vec<Token<'a>>> = Vec::new();
        let mut current_page: Vec<Token<'a>> = Vec::new();

        for token in tokens {
            match token {
                Token::PageBreak(_) => {
                    if !current_page.is_empty() {
                        pages.push(current_page);
                        current_page = Vec::new();
                    }
                }
                _ => {}
            }
            current_page.push(token);
        }

        if !current_page.is_empty() {
            pages.push(current_page);
        }

        Document { pages }
    }
}
