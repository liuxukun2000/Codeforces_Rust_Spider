pub struct UserPool {
    pub users: Vec<User>
}

pub struct User {
    pub username: String,
    pub password: String,
    pub csrf_token: Option<String>,
    is_login: bool,
}

impl User {
    pub fn new(username: &String, password: &String) -> Self {
        Self {
            username: username.clone(),
            password: password.clone(),
            csrf_token: None,
            is_login: false
        }
    }

    pub async fn login(&mut self) -> Result<(), String> {
        if self.is_login == true {
            return Ok(());
        }
        Ok(())
    }
}