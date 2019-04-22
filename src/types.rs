use MalVal::*;

#[derive(Debug)]
pub enum MalVal {
    Int(i32),
    Bool(bool),
    List(Vec<MalVal>),
    Vector(Vec<MalVal>),
    HashMap(Vec<MalVal>),
    Sym(String),
    Str(String),
    Nil,
}

impl ToString for MalVal {
    fn to_string(&self) -> String {
        match self {
            Int(k) => k.to_string(),
            Bool(bool) => bool.to_string(),
            List(l) => print_sequence(&l, "(", ")"),
            Vector(l) => print_sequence(&l, "[", "]"),
            HashMap(l) => print_sequence(&l, "{", "}"),
            Sym(s) => s.clone(),
            Str(s) => s.clone(),
            Nil => "nil".to_owned(),
        }
    }
}

fn print_sequence(seq: &[MalVal], start: &str, end: &str) -> String {
    let seq: Vec<String> = seq
        .iter()
        .map(ToString::to_string)
        .collect();

    format!("{}{}{}", start, seq.join(" "), end)
}

