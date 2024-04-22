use crate::jwt::{JWT_HEADER_ALG, JWT_HEADER_TYP};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct JwtHeader<'a> {
    #[serde(borrow)]
    pub typ: &'a str,
    #[serde(borrow)]
    pub alg: &'a str,
}

impl Default for JwtHeader<'_> {
    fn default() -> Self {
        Self { typ: JWT_HEADER_TYP, alg: JWT_HEADER_ALG }
    }
}

impl<'a> JwtHeader<'a> {
    pub fn is_valid(&self) -> bool {
        self.typ == JWT_HEADER_TYP && self.alg == JWT_HEADER_ALG
    }
}
