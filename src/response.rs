#[derive(Debug, RustcDecodable)]
pub struct Dependency {
    pub head: Vec<isize>,
    pub role: Vec<String>,
    pub tag: Vec<String>,
    pub word: Vec<String>,
}

#[derive(Debug, RustcDecodable)]
pub struct NamedEntity {
    pub entity: Vec<(usize, usize, String)>,
    pub tag: Vec<String>,
    pub word: Vec<String>,
}

#[derive(Debug, RustcDecodable)]
pub struct Tag {
    pub tag: Vec<String>,
    pub word: Vec<String>,
}
