use crate::object::Obj::string;

#[derive(Debug, Clone, PartialEq)]
pub enum Obj {
    string(String),
}

impl Obj {
    pub fn string(str: String) -> Self {
        string(str)
    }

    pub fn print_obj(&self) {
        match self {
            string(v) => print!("{:?}", self),
        }
    }

    pub fn is_string(&self) -> bool {
        match self {
            string(_) => true,
            _ => false,
        }
    }

    pub fn string_val(&self) -> &str {
        match self {
            string(v) => v,
            _ => "",
        }
    }
}
