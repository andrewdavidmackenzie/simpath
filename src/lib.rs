#![deny(missing_docs)]
//! Simpath - or Simple Path is a small library for creating, manipulating and using Unix style
//! `Path`s.
//!
//! A `Path` is an environment variable (a String) with one or more directories specified.
//! They are usually used to find a file that resides in one of the directories.
//! On most platform the default separator character is `:` but on Windows it is `;`
//!
//! If you wish to separate entries with a different separator, it can be modified via API.
//!
#[cfg(feature = "urls")]
extern crate curl;
#[cfg(feature = "urls")]
extern crate url;

use std::env;
use std::fmt;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

#[cfg(feature = "urls")]
use curl::easy::{Easy2, Handler, WriteError};
#[cfg(feature = "urls")]
use url::Url;

#[cfg(feature = "urls")]
struct Collector(Vec<u8>);

#[cfg(feature = "urls")]
impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> std::result::Result<usize, WriteError> {
        self.0.extend_from_slice(data);
        Ok(data.len())
    }
}

// Character used to separate directories in a Path Environment variable on windows is ";"
#[cfg(target_family = "windows")]
const DEFAULT_SEPARATOR_CHAR: char = ';';
// Character used to separate directories in a Path Environment variable on linux/mac/unix is ":"
#[cfg(not(target_family = "windows"))]
const DEFAULT_SEPARATOR_CHAR: char = ':';

/// `Simpath` is the struct returned when you create a new on using a named environment variable
/// which you then use to interact with the `Simpath`
#[derive(Clone, Debug)]
pub struct Simpath {
    separator: char,
    name: String,
    directories: Vec<PathBuf>,
    #[cfg(feature = "urls")]
    urls: Vec<Url>,
}

/// `FileType` can be used to find an entry in a path of a specific type (`Directory`, `File`, `URL`)
/// or of `Any` type
#[derive(Debug, PartialEq)]
pub enum FileType {
    /// An entry in the `Simpath` of type `File`
    File,
    /// An entry in the `Simpath` of type `Directory`
    Directory,
    /// An entry in the `Simpath` of type `Url`
    Resource,
    /// An entry in the `Simpath` of `Any` types
    Any,
}

/// `FoundType` indicates what type of entry was found
#[derive(Debug, PartialEq)]
pub enum FoundType {
    /// An entry in the `Simpath` of type `File`
    File(PathBuf),
    /// An entry in the `Simpath` of type `Directory`
    Directory(PathBuf),
    #[cfg(feature = "urls")]
    /// An entry in the `Simpath` of type `Url`
    Resource(Url),
}

/// When validating a `Simpath` there can be the following types of `PathError`s returned
pub enum PathError {
    /// The `Path` entry does not exist on the file system
    DoesNotExist(String),
    /// The `Path` entry cannot be reads
    CannotRead(String),
}

impl Simpath {
    /// Create a new simpath, providing the name of the environment variable to initialize the
    /// search path with. If an environment variable of that name exists and it will be parsed
    /// as a ':' separated list of paths to search. Only paths detected as directories will
    /// be used, not files.
    ///
    /// If an environment variable of that name is *not* found, a new simpath will be created anyway
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
    ///         Ok(found) => println!("'ls' was found at '{:?}'", found),
    ///         Err(e)   => println!("{}", e)
    ///     }
    /// }
    /// ```
    ///
    pub fn new(var_name: &str) -> Self {
        let mut search_path = Simpath {
            separator: DEFAULT_SEPARATOR_CHAR,
            name: var_name.to_string(),
            directories: Vec::<PathBuf>::new(),
            #[cfg(feature = "urls")]
            urls: Vec::<Url>::new(),
        };

        search_path.add_from_env_var(var_name);

        search_path
    }

    /// Create a new simpath, providing the name of the environment variable to initialize the
    /// search path with and the separator character for this search path to be used from here on.
    /// If an environment variable of that name exists and it will be parsed as a list of paths to
    /// search. Only paths detected as directories will be used, not files.
    ///
    /// If an environment variable of that name is *not* found, a new simpath will be created anyway
    /// and it can have directories added to it programatically and used in the normal fashion to
    /// search for files.
    ///
    /// In all cases, the separator char for this search path will be set to `separator` from here on.
    ///
    /// ```
    /// extern crate simpath;
    /// use simpath::Simpath;
    /// use std::env;
    ///
    /// fn main() {
    ///     env::set_var("TEST", "/,.,~");
    ///     let search_path = Simpath::new("TEST");
    ///     let two = search_path.find(".");
    ///     match two {
    ///         Ok(found) => println!("'.' was found at '{:?}'", found),
    ///         Err(e)   => println!("{}", e)
    ///     }
    /// }
    /// ```
    pub fn new_with_separator(var_name: &str, separator: char) -> Self {
        let mut search_path = Simpath {
            separator,
            name: var_name.to_string(),
            directories: Vec::<PathBuf>::new(),
            #[cfg(feature = "urls")]
            urls: Vec::<Url>::new(),
        };

        search_path.add_from_env_var(var_name);

        search_path
    }

    /// Get the currently set separator character that is used when parsing entries from an environment
    /// variable
    pub fn separator(&self) -> char {
        self.separator
    }

    /// Get the name associated with the simpath. Note that this could be an empty String
    /// ```
    /// extern crate simpath;
    /// use simpath::Simpath;
    ///
    /// fn main() {
    ///     let search_path = Simpath::new("PATH");
    ///     println!("Directories in Search Path: {:?}", search_path.name());
    /// }
    /// ```
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
    pub fn directories(&self) -> &Vec<PathBuf> {
        &self.directories
    }

    #[cfg(feature = "urls")]
    /// Get the list of URLs that are included in the Search Path
    ///
    /// ```
    /// extern crate simpath;
    /// use simpath::Simpath;
    /// use std::env;
    ///
    /// fn main() {
    ///     env::set_var("TEST", "http://ibm.com,https://hp.com");
    ///     let search_path = Simpath::new("TEST");
    ///     println!("URLs in Search Path: {:?}", search_path.urls());
    /// }
    /// ```
    pub fn urls(&self) -> &Vec<Url> {
        &self.urls
    }

    /// Try to find a file or resource by name (not full path) on a search path.
    /// Searching for a file could cause errors, so Result<FoundType, io::Error> is returned
    /// If it is found `Ok(FoundType)` is returned indicating where the resource/file can be found.
    /// If it is not found then `Err` is returned.
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
    pub fn find(&self, file_name: &str) -> Result<FoundType, Error> {
        self.find_type(file_name, FileType::Any)
    }

    /// find an entry of a specific `FileType` in a `Path`
    ///
    /// ```
    /// extern crate simpath;
    /// use simpath::Simpath;
    ///
    /// fn main() {
    ///     use simpath::FileType;
    ///     let search_path = Simpath::new("PATH");
    ///     match search_path.find_type("my-file", FileType::Directory) {
    ///         Ok(_found_dir) => println!("Didn't expect that!!"),
    ///         Err(e)         => println!("{}", e.to_string())
    ///     }
    /// }
    /// ```
    pub fn find_type(&self, file_name: &str, file_type: FileType) -> Result<FoundType, Error> {
        if file_type == FileType::File || file_type == FileType::Directory || file_type == FileType::Any {
            for search_dir in &self.directories {
                for entry in fs::read_dir(search_dir)? {
                    let file = entry?;
                    if let Some(filename) = file.file_name().to_str() {
                        if filename == file_name {
                            let metadata = file.metadata()?;
                            match file_type {
                                FileType::Any => return Ok(FoundType::File(file.path())),
                                FileType::Directory if metadata.is_dir() => return Ok(FoundType::Directory(file.path())),
                                FileType::File if metadata.is_file() => return Ok(FoundType::File(file.path())),
                                _ => { /* keep looking */ }
                            }
                        }
                    }
                }
            }
        }

        #[cfg(feature = "urls")]
        if file_type == FileType::Resource || file_type == FileType::Any {
            for base_url in &self.urls {
                let url = base_url.join(file_name)
                    .map_err(|e| Error::new(ErrorKind::NotFound, e.to_string()))?;
                if Self::resource_exists(&url).is_ok() {
                    return Ok(FoundType::Resource(url));
                }
            }
        }

        Err(Error::new(ErrorKind::NotFound,
                       format!("Could not find type '{:?}' called '{}' in search path '{}'",
                               file_type, file_name, self.name)))
    }

    #[cfg(feature = "urls")]
    fn resource_exists(url: &Url) -> Result<(), Error> {
        let mut easy = Easy2::new(Collector(Vec::new()));
        easy.nobody(true)?;

        easy.url(&url.to_string())?;
        easy.perform()?;
        let response_code = easy.response_code()
            .map_err(|e| Error::new(ErrorKind::NotFound, e.to_string()))?;

        // Consider 301 - Permanently Moved as the resource NOT being at this Url
        // An option to consider is asking the request library to follow the redirect.
        match response_code {
            200..=299 => Ok(()),
            error_code => Err(Error::new(ErrorKind::NotFound,
                                         format!("Response code: {} from '{}'", error_code, url)))
        }
    }

    /// Add an to the search path.
    ///
    /// if "urls" feature is enabled:
    ///     If it parses as as web Url it will be added to the list of
    ///     base Urls to search, otherwise it will be added to the list of directories to search.
    /// if "urls" feature is *not* enabled:
    ///     It is assumed to be a directory and added using `add_directory()`
    ///
    /// ```
    /// extern crate simpath;
    /// use simpath::Simpath;
    ///
    /// fn main() {
    ///     let mut search_path = Simpath::new("PATH");
    ///     search_path.add(".");
    ///
    /// #[cfg(feature = "urls")]
    ///     search_path.add("http://ibm.com");
    ///
    ///     println!("{}", search_path);
    /// }
    /// ```
    pub fn add(&mut self, entry: &str) {
        #[cfg(not(feature = "urls"))]
            self.add_directory(entry);

        #[cfg(feature = "urls")]
        match Url::parse(entry) {
            Ok(url) => {
                match url.scheme() {
                    #[cfg(feature = "urls")]
                    "http" | "https" => self.add_url(&url),
                    "file" => self.add_directory(url.path()),
                    _ => self.add_directory(entry)
                }
            }
            Err(_) => self.add_directory(entry) /* default to being a directory path */
        }
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
    pub fn add_directory(&mut self, dir: &str) {
        let path = PathBuf::from(dir);
        if path.exists() && path.is_dir() && path.read_dir().is_ok() {
            if let Ok(canonical) = path.canonicalize() {
                self.directories.push(canonical);
            }
        }
    }

    #[cfg(feature = "urls")]
    /// Add a Url to the list of Base Urls to be used when searching for resources.
    ///
    /// ```
    /// extern crate simpath;
    /// extern crate url;
    ///
    /// use simpath::Simpath;
    /// use url::Url;
    ///
    /// fn main() {
    ///     let mut search_path = Simpath::new("WEB");
    ///     search_path.add_url(&Url::parse("http://ibm.com").unwrap());
    ///     println!("Urls in Search Path: {:?}", search_path.urls());
    /// }
    /// ```
    pub fn add_url(&mut self, url: &Url) {
        self.urls.push(url.clone());
    }

    /// Check if a search path contains an entry
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
    pub fn contains(&self, entry: &str) -> bool {
        #[cfg(not(feature = "urls"))]
            return self.directories.contains(&PathBuf::from(entry));

        #[cfg(feature = "urls")]
        if self.directories.contains(&PathBuf::from(entry)) {
            true
        } else {
            if let Ok(url_entry) = Url::parse(entry) {
                return self.urls.contains(&url_entry);
            }
            false
        }
    }

    /// Add entries to the search path, by reading them from an environment variable.
    ///
    /// The environment variable should have a set of entries separated by the separator character.
    /// By default the separator char is `":"` (on non-windows platforms) and `";"` (on windows)
    /// but it can be modified after creation of search path.
    ///
    /// The environment variable is parsed using the separator char set at the time this function
    /// is called.
    ///
    /// To be added each entry must exist and be readable.
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
    pub fn add_from_env_var(&mut self, var_name: &str) {
        if let Ok(var_string) = env::var(var_name) {
            for part in var_string.split(self.separator) {
                self.add(part);
            }
        }
    }

    /// Add entries to the search path, by reading them from an environment variable.
    ///
    /// The environment variable should have a set of entries separated by the specified
    /// separator character.
    ///
    /// To be added each entry must exist and be readable.
    ///
    /// NOTE: The separator char is only used while parsing the specified environment variable and
    /// *does not* modify the separator character in use in the Simpath after this function completes.
    ///
    /// ```
    /// extern crate simpath;
    /// use simpath::Simpath;
    /// use std::env;
    ///
    /// fn main() {
    ///     let mut search_path = Simpath::new("MyPathName");
    ///     env::set_var("TEST", "/,.,~");
    ///     search_path.add_from_env_var_with_separator("TEST", ',');
    ///     if search_path.contains(".") {
    ///         println!("'.' was in your 'TEST' environment variable and has been added to the search path called '{}'",
    ///                  search_path.name());
    ///     }
    /// }
    /// ```
    pub fn add_from_env_var_with_separator(&mut self, var_name: &str, separator: char) {
        if let Ok(var_string) = env::var(var_name) {
            for part in var_string.split(separator) {
                self.add_directory(part);
            }
        }
    }
}

impl fmt::Display for Simpath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Search Path '{}': Directories: {:?}", self.name, self.directories)?;

        #[cfg(feature = "urls")]
        write!(f, ", URLs: {:?}", self.urls)?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::env;
    use std::fs;
    use std::io::Write;

    use ::{DEFAULT_SEPARATOR_CHAR, FileType};

    use super::Simpath;

    #[test]
    fn can_create() {
        Simpath::new("PATH");
    }

    #[test]
    fn can_create_with_separator() {
        Simpath::new_with_separator("PATH", ':');
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
    fn display_empty_path() {
        let path = Simpath::new("MyName");
        println!("{}", path);
    }

    #[test]
    fn directory_is_added() {
        let mut path = Simpath::new("MyName");
        assert!(path.directories().is_empty());
        path.add_directory(".");
        let cwd = env::current_dir()
            .expect("Could not get current working directory").to_string_lossy().to_string();
        println!("cwd = {}", cwd);
        println!("path = {}", path);
        assert!(path.contains(&cwd));
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
        let temp_dir = tempdir::TempDir::new("simpath").unwrap().into_path();
        let mut parent_dir = temp_dir.clone();
        parent_dir.pop();

        // Create a ENV path that includes that dir
        let var_name = "MyPath";
        env::set_var(var_name, &parent_dir);

        // create a simpath from the env var
        let path = Simpath::new(var_name);

        // Check that simpath can find the temp_dir
        let temp_dir_name = format!("{}.{}",
                                    temp_dir.file_stem().unwrap().to_str().unwrap(),
                                    temp_dir.extension().unwrap().to_str().unwrap());
        assert!(path.find_type(&temp_dir_name, FileType::Directory).is_ok(),
                "Could not find the simpath temp directory in Path set from env var");

        // clean-up
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn find_file_from_env_variable() {
        // Create a temp dir for test
        let temp_dir = tempdir::TempDir::new("simpath").unwrap().into_path();

        // Create a ENV path that includes the path to the temp dir
        let var_name = "MYPATH";
        env::set_var(var_name, &temp_dir);

        // create a simpath from the env var
        let path = Simpath::new(var_name);

        // Create a file in the directory
        let temp_filename = "testfile";
        let temp_file_path = format!("{}/{}", temp_dir.display(), temp_filename);
        let mut file = fs::File::create(&temp_file_path).unwrap();
        file.write_all(b"test file contents").unwrap();

        // Check that simpath can find the file
        assert!(path.find_type(temp_filename, FileType::File).is_ok(),
                "Could not find 'testfile' in Path set from env var");

        // clean-up
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn find_dir_using_any_from_env_variable() {
        // Create a temp dir for test
        let temp_dir = tempdir::TempDir::new("simpath").unwrap().into_path();

        // Create a ENV path that includes that dir
        let var_name = "MyPath";
        env::set_var(var_name, &temp_dir);

        // create a simpath from the env var
        let path = Simpath::new(var_name);

        // Create a file in the directory
        let temp_filename = "testfile";
        let temp_file_path = format!("{}/{}", temp_dir.display(), temp_filename);
        let mut file = fs::File::create(&temp_file_path).unwrap();
        file.write_all(b"test file contents").unwrap();

        // Check that simpath can find it
        assert!(path.find(temp_filename).is_ok(),
                "Could not find the 'testfile' in Path set from env var");

        // clean-up
        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn single_add_from_env_variable() {
        let var_name = "MyPath";
        env::set_var(var_name, ".");
        let path = Simpath::new(var_name);
        assert!(path.contains(&env::current_dir()
            .expect("Could not get current working directory").to_string_lossy().to_string()));
    }

    #[test]
    fn multiple_add_from_env_variable() {
        let var_name = "MyPath";
        env::set_var(var_name, format!(".{}/", DEFAULT_SEPARATOR_CHAR));
        let path = Simpath::new(var_name);
        assert!(path.contains(&env::current_dir()
            .expect("Could not get current working directory").to_string_lossy().to_string()));
        assert!(path.contains("/"));
    }

    #[test]
    fn multiple_add_from_env_variable_separator() {
        let var_name = "MyPath";
        env::set_var(var_name, ".,/");
        let path = Simpath::new_with_separator(var_name, ',');
        assert!(path.contains(&env::current_dir()
            .expect("Could not get current working directory").to_string_lossy().to_string()));
        assert!(path.contains("/"));
    }

    #[test]
    fn display_a_simpath_with_entries() {
        let var_name = "MyPath";
        env::set_var(var_name, format!(".{}/", DEFAULT_SEPARATOR_CHAR));
        let path = Simpath::new(var_name);
        println!("Simpath can be printed: {}", path);
    }

    #[test]
    fn entry_does_not_exist() {
        let var_name = "MyPath";
        env::set_var(var_name, "/foo");
        let path = Simpath::new(var_name);
        assert_eq!(path.directories().len(), 0);
    }

    #[test]
    fn one_entry_does_not_exist() {
        let var_name = "MyPath";
        env::set_var(var_name, format!(".{}/foo", DEFAULT_SEPARATOR_CHAR));
        let path = Simpath::new(var_name);
        assert_eq!(path.directories().len(), 1);
        assert!(!path.contains("/foo"));
    }

    #[cfg(feature = "urls")]
    mod url_tests {
        use std::env;
        use url::Url;
        use FileType;
        use super::Simpath;

        const BASE_URL: &str = "https://www.ibm.com";
        const EXISTING_RESOURCE: &str = "/es-es";

        #[test]
        fn create_from_env() {
            let var_name = "MyPath";
            env::set_var(var_name, BASE_URL);
            let path = Simpath::new_with_separator(var_name, ',');
            assert_eq!(path.urls().len(), 1);
            assert_eq!(path.directories().len(), 0);
            assert!(path.urls().contains(&Url::parse(BASE_URL)
                .expect("Could not parse URL")));
        }

        #[test]
        fn add_url_that_exists() {
            let mut path = Simpath::new_with_separator("test", ',');
            path.add_url(&Url::parse(BASE_URL)
                .expect("Could not parse Url"));
            assert_eq!(path.urls().len(), 1);
            assert_eq!(path.directories().len(), 0);
            assert!(path.urls().contains(&Url::parse(BASE_URL)
                .expect("Could not parse URL")));
        }

        #[test]
        fn find_resource_not_exist() {
            let mut search_path = Simpath::new("TEST");
            search_path.add_url(&Url::parse(BASE_URL).expect("Could not parse Url"));
            assert!(search_path.find_type("/no-way-this-exists", FileType::Resource).is_err(),
                    "should not find the resource");
        }

        #[test]
        fn find_existing_resource() {
            let mut search_path = Simpath::new("TEST");
            search_path.add_url(&Url::parse(BASE_URL).expect("Could not parse Url"));
            search_path.find_type(EXISTING_RESOURCE, FileType::Resource).expect("Could not find '/'");
        }

        #[test]
        fn contains_url_that_exists() {
            let var_name = "MyPath";
            env::set_var(var_name, BASE_URL);
            let path = Simpath::new_with_separator(var_name, ',');
            assert!(path.contains(BASE_URL));
        }

        #[test]
        fn display_path_with_directory_and_url() {
            let var_name = "MyPath";
            env::set_var(var_name, format!("~,{}", BASE_URL));
            let path = Simpath::new_with_separator(var_name, ',');
            println!("{}", path);
        }
    }
}