use super::Type;

pub fn join(v: &Vec<Type>) -> String {
    v.iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}