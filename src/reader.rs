use pom::parser::*;
use pom::char_class::*;

use std::collections::HashMap;

use crate::types::*;

fn spaces<'a>() -> Parser<'a, u8, ()> {
    one_of(b" \n\r,").repeat(0..).discard()
}

fn comment<'a>() -> Parser<'a, u8, ()> {
    sym(b';') * none_of(b"\r\n").discard()
}

fn ignored<'a>() -> Parser<'a, u8, ()> {
    spaces() | comment()
}

fn symbol(term: u8) -> bool {
    b"!#$%&|*+-/:<=>?_".contains(&term)
}

fn escaped<'a>() -> Parser<'a, u8, u8> {
    let p = sym(b'\\') * one_of(b"\\\"n");
    
    p.map(|v| match v {
        b'n' => b'\n',
        _ => v,
    })
}

pub fn read_form<'a>() -> Parser<'a, u8, MalVal> {
    ignored() * (read_atom() | read_list() | read_macro() | read_vector() | read_hashmap())
}

fn delimited<'a, T>(
    start: Parser<'a, u8, T>, 
    end: Parser<'a, u8, T>,
    elem: Parser<'a, u8, MalVal>) -> Parser<'a, u8, Vec<MalVal>> 
where 
    T: 'a,
{
    start * ignored() * list(elem, ignored()) - ignored() - end
}

fn read_list<'a>() -> Parser<'a, u8, MalVal> {
    delimited(sym(b'('), sym(b')'), call(read_form)).map(MalVal::List)
}

fn read_vector<'a>() -> Parser<'a, u8, MalVal> {
    delimited(sym(b'['), sym(b']'), call(read_form)).map(MalVal::Vector)
}

// TODO: Refactor hashmap reader. Get rid of unwraps
fn read_hashmap<'a>() -> Parser<'a, u8, MalVal> {
    (sym(b'{') * ignored() * list(call(read_form), ignored()) - ignored() - sym(b'}'))
        .map(|mut v| pair_list(&mut v).unwrap())
        .map(MalVal::HashMap)
}

fn pair_list(list: &mut Vec<MalVal>) -> Result<HashMap<String, MalVal>> {
    let mut hm = HashMap::new();

    while !list.is_empty() {
        let v = list.pop().unwrap();

        match list.pop().unwrap() {
            MalVal::Str(k) => hm.insert(k, v),
            MalVal::Sym(k) => hm.insert(k, v),
            _ => return Err(Error::ParseError),
        };
    }

    Ok(hm)
}

fn read_atom<'a>() -> Parser<'a, u8, MalVal> {
    read_number() | read_symbol() | read_string()
}

fn read_number<'a>() -> Parser<'a, u8, MalVal> {
    let p = sym(b'-').opt() + one_of(b"1234567890").repeat(1..);

    p.collect()
     .map(|k| k.to_vec() )
     .convert(String::from_utf8)
     .convert(|k| k.parse())
     .map(MalVal::Int)
}

fn read_metadata<'a>() -> Parser<'a, u8, MalVal> {
    let p = sym(b'^') * call(read_form) + call(read_form);
    p.map(|(mv1, mv2)| {
        let v = vec![MalVal::Sym("with-meta".to_string()), mv2, mv1];
        MalVal::List(v)
    })
}

fn read_macro<'a>() -> Parser<'a, u8, MalVal> {
    read_splice_unquote() 
        | read_unquote() 
        | read_quote() 
        | read_quasiquote() 
        | read_deref() 
        | read_metadata()
}

fn read_quote<'a>() -> Parser<'a, u8, MalVal> {
    let p = sym(b'\'') * call(read_form);
    p.map(|mv| {
        let v = vec![MalVal::Sym("quote".to_string()), mv];
        MalVal::List(v)
    })
}

fn read_deref<'a>() -> Parser<'a, u8, MalVal> {
    let p = sym(b'@') * call(read_form);
    p.map(|mv| {
        let v = vec![MalVal::Sym("deref".to_string()), mv];
        MalVal::List(v)
    })
}

fn read_quasiquote<'a>() -> Parser<'a, u8, MalVal> {
    let p = sym(b'`') * call(read_form);
    p.map(|mv| {
        let v = vec![MalVal::Sym("quasiquote".to_string()), mv];
        MalVal::List(v)
    })
}

fn read_unquote<'a>() -> Parser<'a, u8, MalVal> {
    let p = sym(b'~') * call(read_form);
    p.map(|mv| {
        let v = vec![MalVal::Sym("unquote".to_string()), mv];
        MalVal::List(v)
    })
}

fn read_splice_unquote<'a>() -> Parser<'a, u8, MalVal> {
    let p = sym(b'~') * sym(b'@') * call(read_form);
    p.map(|mv| {
        let v = vec![MalVal::Sym("splice-unquote".to_string()), mv];
        MalVal::List(v)
    })
}

fn read_string<'a>() -> Parser<'a, u8, MalVal> {
    let p = sym(b'\"') * (escaped() | none_of(b"\"")).repeat(0..) - sym(b'\"');

    p.collect()
     .map(|k| k.to_vec() )
     .convert(String::from_utf8)
     .map(MalVal::Str)
}

fn read_symbol<'a>() -> Parser<'a, u8, MalVal> {
    let p = (is_a(symbol) | is_a(alpha)) + (is_a(symbol) | is_a(alphanum)).repeat(0..);

    p.collect()
     .map(|k| k.to_vec() )
     .convert(String::from_utf8)
     .map(|s| {
         match s.as_ref() {
             "true" => MalVal::Bool(true),
             "false" => MalVal::Bool(false),
             "nil" => MalVal::Nil,
             _ => MalVal::Sym(s),
         }
     })
}
