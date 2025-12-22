// Ejemplo de código Rust para pruebas
use std::collections::HashMap;

/// Estructura que representa un usuario
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
}

impl User {
    /// Crea un nuevo usuario
    pub fn new(id: u64, name: String, email: String) -> Self {
        Self { id, name, email }
    }
    
    /// Valida el email del usuario
    pub fn is_valid_email(&self) -> bool {
        self.email.contains('@') && self.email.contains('.')
    }
}

/// Gestiona una colección de usuarios
pub struct UserManager {
    users: HashMap<u64, User>,
}

impl UserManager {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }
    
    pub fn add_user(&mut self, user: User) {
        self.users.insert(user.id, user);
    }
    
    pub fn get_user(&self, id: u64) -> Option<&User> {
        self.users.get(&id)
    }
}
