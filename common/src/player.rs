#[derive(Clone, Debug)]
pub struct Player {
    id: u8,
    name: String
}

impl Player {
    pub fn new(id: u8, name: String) -> Self {
        Self { id, name }
    }

    pub fn id(&self) -> u8 {
        self.id
    }

    pub fn name(&self) -> &String {
        &self.name
    }
}