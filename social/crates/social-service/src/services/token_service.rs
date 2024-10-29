use std::sync::Arc;

use crate::repositories::token_repository::TokenRepository;

pub struct TokenService {
    token_repository: Arc<TokenRepository>,
}

impl TokenService {
    pub fn new(token_repository: Arc<TokenRepository>) -> Self {
        Self { token_repository }
    }
}
