pub struct Simpath {
    pub var_name: String
}

impl Simpath {
    pub fn new(var_name: &str) -> Self {
        Simpath {
            var_name: var_name.to_string()
        }
    }
}

#[cfg(test)]
mod test {
    use super::Simpath;

    #[test]
    fn can_create() {
        Simpath::new("PATH");
    }
}