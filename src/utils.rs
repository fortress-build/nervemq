pub fn to_pom_error<E: std::fmt::Display>(e: E, position: usize, msg: &'static str) -> pom::Error {
    pom::Error::Conversion {
        message: format!("{}: {}", msg, e),
        position,
    }
}
