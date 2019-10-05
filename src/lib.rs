use std::path::PathBuf;
use std::fmt;
use std::env;
use std::fs;
use std::io::{Error, ErrorKind};

#[derive(Clone, Debug)]
pub struct Simpath {
    name: String,
    dirs: Vec<PathBuf>
}

#[derive(Debug)]
pub enum FileType {
    File,
    Directory,
    Any
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
    ///         Ok(path) => println!("'ls' was found at '{}'", path.display()),
    ///         Err(e)   => println!("{}", e)
    ///     }
    /// }
    /// ```
    ///
    pub fn new(var_name: &str) -> Self {
        let mut search_path = Simpath {
            name: var_name.to_string(),
            dirs: vec!()
        };

        search_path.add_from_env_var(var_name);

        search_path
    }

    /// Get the name associated with the simpath. Note that this could be an empty String
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the list of directories that are included in the Search Path
    ///
    /// ```
    /// extern crate simpath;
    /// use simpath::Simpath;
    ///
    /// fn main() {
    ///     let search_path = Simpath::new("PATH");
    ///     println!("Directories in Search Path: {:?}", search_path.directories());
    /// }
    /// ```
    ///
    pub fn directories(&self) -> &Vec<PathBuf> {
        &self.dirs
    }

    /// Try to find a file by filename (not full path) on a search path.
    /// Searching for a file could cause errors, so Result<PathBuf, io::Error> is returned
    /// If it is found `Ok(PathBuf)` path to the file will be returned.
    /// If it is not found then `Err is returned.`
    ///
    /// ```
    /// extern crate simpath;
    /// use simpath::Simpath;
    ///
    /// fn main() {
    ///     let search_path = Simpath::new("PATH");
    ///     match search_path.find("my-file") {
    ///         Ok(_found_dir) => println!("Didn't expect that!!"),
    ///         Err(e)         => println!("{}", e.to_string())
    ///     }
    /// }
    /// ```
    ///
    pub fn find(&self, file_name: &str) -> Result<PathBuf, Error> {
        self.find_type(file_name, FileType::Any)
    }

    pub fn find_type(&self, file_name: &str, file_type: FileType) -> Result<PathBuf, Error> {
        for search_dir in &self.dirs {
            for entry in fs::read_dir(search_dir)? {
                let file = entry?;
                if let Some(filename) = file.file_name().to_str() {
                    if filename  == file_name {
                        let metadata = file.metadata()?;
                        match file_type {
                            FileType::Any => return Ok(file.path()),
                            FileType::Directory if metadata.is_dir() => return Ok(file.path()),
                            FileType::File if metadata.is_file() => return Ok(file.path()),
                            _ => { /* keep looking */}
                        }
                    }
                }
            }
        }

        Err(Error::new(ErrorKind::NotFound,
                       format!("Could not find type '{:?}' called '{}' in search path '{}'",
                               file_type, file_name, self.name)))
    }

    /// Add a directory to the list of directories to search for files.
    /// If the directory passed does not exist, or is not a directory, or cannot be read then it
    /// will be ignored.
    ///
    /// ```
    /// extern crate simpath;
    /// use simpath::Simpath;
    ///
    /// fn main() {
    ///     let mut search_path = Simpath::new("PATH");
    ///     search_path.add_directory(".");
    ///     println!("Directories in Search Path: {:?}", search_path.directories());
    /// }
    /// ```
    ///
    pub fn add_directory(&mut self, dir: &str) {
        let path = PathBuf::from(dir);
        if path.exists() && path.is_dir() && path.read_dir().is_ok() {
            self.dirs.push(path);
        }
    }

    /// Check if a search path contains a directory
    ///
    ///
    /// ```
    /// extern crate simpath;
    /// use simpath::Simpath;
    ///
    /// fn main() {
    ///     let mut search_path = Simpath::new("FakeEnvVar");
    ///     if search_path.contains(".") {
    ///         println!("Well that's a surprise!");
    ///     }
    /// }
    /// ```
    pub fn contains(&self, dir: &str) -> bool {
        for search_dir in &self.dirs {
            if search_dir.to_str().unwrap() == dir {
                return true;
            }
        }
        false
    }

    /// Add entries to the search path, by reading from an environment variable.
    /// The variable should have a set of ':' separated directory names.
    /// To be added each direcory should exist and be readable.
    ///
    /// ```
    /// extern crate simpath;
    /// use simpath::Simpath;
    ///
    /// fn main() {
    ///     let mut search_path = Simpath::new("MyPathName");
    ///     search_path.add_from_env_var("PATH");
    ///     if search_path.contains(".") {
    ///         println!("'.' was in your 'PATH' and has been added to the search path called '{}'",
    ///                  search_path.name());
    ///     }
    /// }
    /// ```
    ///
    pub fn add_from_env_var(&mut self, var_name: &str) {
        if let Ok(var_string) = env::var(var_name) {
            for part in var_string.split(":") {
                self.add_directory(part);
            }
        }
    }
}

impl fmt::Display for Simpath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Search Path: '{}', Directories: {{", self.name).unwrap();
        for dir in &self.dirs {
            write!(f, "'{}', ", dir.display()).unwrap();

        }
        write!(f, "}}").unwrap();
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::Simpath;
    use std::env;
    use std::fs;
    use std::io::Write;
    use FileType;

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
        assert!(path.find("no_such_file").is_err());
    }

    #[test]
    fn display_path() {
        let path = Simpath::new("MyName");
        println!("{}", path);
    }

    #[test]
    fn directory_is_added() {
        let mut path = Simpath::new("MyName");
        assert!(path.directories().is_empty());
        path.add_directory(".");
        assert!(path.contains("."))
    }

    #[test]
    fn cant_add_non_dir() {
        let mut path = Simpath::new("MyName");
        assert!(path.directories().is_empty());
        path.add_directory("no-such-dir");
        assert_eq!(path.contains("no-such-dir"), false);
    }

    #[test]
    fn find_dir_from_env_variable() {
        // Create a temp dir for test
        let temp_dir= tempdir::TempDir::new("simpath").unwrap().into_path();
        let mut parent_dir = temp_dir.clone();
        parent_dir.pop();

        // Create a ENV path that includes that dir
        let var_name = "MyPathEnv";
        env::set_var(var_name, &parent_dir);

        // create a simpath from the env var
        let path = Simpath::new(var_name);

        // Check that simpath can find the temp_dir
        let temp_dir_name = format!("{}.{}",
                                    temp_dir.file_stem().unwrap().to_str().unwrap(),
                                    temp_dir.extension().unwrap().to_str().unwrap());
        assert!(path.find_type(&temp_dir_name, FileType::Directory).is_ok(),
                "Could not find the directory '.' in Path set from env var");

        // clean-up
        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn find_file_from_env_variable() {
        // Create a temp dir for test
        let temp_dir= tempdir::TempDir::new("simpath").unwrap().into_path();

        // Create a ENV path that includes that dir
        let var_name = "MyPathEnv";
        env::set_var(var_name, &temp_dir);

        // create a simpath from the env var
        let path = Simpath::new(var_name);

        // Create a file in the directory
        let temp_filename = "simpath.test";
        let temp_file_path = format!("{}/{}", temp_dir.display(), temp_filename);
        let mut file = fs::File::create(&temp_file_path).unwrap();
        file.write_all(b"test file contents").unwrap();

        // Check that simpath can find it
        assert!(path.find_type(temp_filename, FileType::File).is_ok(),
                "Could not find the directory '.' in Path set from env var");

        // clean-up
        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn find_any_from_env_variable() {
        // Create a temp dir for test
        let temp_dir= tempdir::TempDir::new("simpath").unwrap().into_path();

        // Create a ENV path that includes that dir
        let var_name = "MyPathEnv";
        env::set_var(var_name, &temp_dir);

        // create a simpath from the env var
        let path = Simpath::new(var_name);

        // Create a file in the directory
        let temp_filename = "simpath.test";
        let temp_file_path = format!("{}/{}", temp_dir.display(), temp_filename);
        let mut file = fs::File::create(&temp_file_path).unwrap();
        file.write_all(b"test file contents").unwrap();

        // Check that simpath can find it
        assert!(path.find(temp_filename).is_ok(),
                "Could not find the directory '.' in Path set from env var");

        // clean-up
        fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn single_add_from_env_variable() {
        let var_name = "MyPathEnv";
        env::set_var(var_name, ".");
        let mut path = Simpath::new("MyName");
        path.add_from_env_var(var_name);
        assert!(path.contains("."));
    }

    #[test]
    fn multiple_add_from_env_variable() {
        let var_name = "MyPathEnv";
        env::set_var(var_name, ".:/");
        let mut path = Simpath::new("MyName");
        path.add_from_env_var(var_name);
        assert!(path.contains("."));
        assert!(path.contains("/"));
    }

    #[test]
    fn display_a_simpath() {
        let var_name = "MyPathEnv";
        env::set_var(var_name, ".:/");
        let mut path = Simpath::new("MyName");
        path.add_from_env_var(var_name);

        println!("Simpath can be printed: {}", path);
    }
}