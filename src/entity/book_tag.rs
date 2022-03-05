#[derive(Debug, Clone)]
pub enum BookTag {
    Artist(String),
    Series(String),
    Group(String),
    Character(String),
    Female(String),
    Male(String),
    Misc(String),
}

impl BookTag {
    pub fn kind(&self) -> &str {
        use BookTag::*;

        match self {
            Artist(_) => "artist",
            Series(_) => "series",
            Group(_) => "group",
            Character(_) => "character",
            Female(_) => "female",
            Male(_) => "male",
            Misc(_) => "misc",
        }
    }

    pub fn name(&self) -> &str {
        use BookTag::*;

        match self {
            Artist(name) => name,
            Series(name) => name,
            Group(name) => name,
            Character(name) => name,
            Female(name) => name,
            Male(name) => name,
            Misc(name) => name,
        }
    }
}
