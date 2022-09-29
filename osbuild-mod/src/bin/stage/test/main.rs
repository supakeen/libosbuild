fn main() {
    println!("{{\"foo\": \"bar\"}}");
}

#[cfg(test)]
mod test {
    #[test]
    fn dummy() {
        assert!(true);
    }
}
