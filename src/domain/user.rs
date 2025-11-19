#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone)]
pub struct UserCreate {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone)]
pub struct UserPatch {
    pub name: Option<String>,
    pub email: Option<String>,
}

impl User {
    pub fn new(name: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            id: String::new(),
            name: name.into(),
            email: email.into(),
        }
    }
}
