struct Element {
    name: String,
    attributes: Vec<(String, String)>,
    children: Vec<Element>,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
