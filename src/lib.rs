use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Simpath {
    name: String
}

impl Simpath {
    /// Create a new simpath, providing the name of the environment variable to initialize the
    /// search path with. If an environment variable of that name exists and it will be parsed
    /// as a ':' separated list of paths to search. Only paths detected as directories will
    /// be used, not files.
    ///
    /// If an environment variable of that name is not found, a new simpath will be created anyway
    /// and it can have directories added to it programatically and used in the normal fashion to
    /// search for files
    ///
    /// ```
    /// extern crate simpath;
    /// use simpath::Simpath;
    ///
    /// fn main() {
    ///     let search_path = Simpath::new("PATH");
    ///     let ls_file = search_path.find("ls");
    ///     match ls_file {
    ///         Some(path) => println!("'ls' was found at '{}'", path.display()),
    ///         None       => println!("'ls' was not found on the search path '{}'",
    ///                                 search_path.name())
    ///     }
    /// }
    /// ```
    ///
    pub fn new(var_name: &str) -> Self {
        Simpath {
            name: var_name.to_string()
        }
    }

    /// Get the name associated with the simpath. Note that this could be an empty String
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Try to find a file on the search path. If it is found `Some(PathBuf)` path to the file will
    /// be returned. If it is not found then `None is returned.
    pub fn find(&self, file_name: &str) -> Option<PathBuf> {
        None // TODO
    }
}

#[cfg(test)]
mod test {
    use super::Simpath;

    #[test]
    fn can_create() {
        Simpath::new("PATH");
    }

    #[test]
    fn name_is_saved() {
        let path = Simpath::new("MyName");
        assert_eq!(path.name(), "MyName");
    }

    #[test]
    fn find_non_existant_file() {
        let path = Simpath::new("MyName");
        assert_eq!(path.find("no_such_file"), None);
    }
}