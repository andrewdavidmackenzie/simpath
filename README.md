[![Build Status](https://travis-ci.org/andrewdavidmackenzie/simpath.svg?branch=master)](https://travis-ci.org/andrewdavidmackenzie/flow)

# simpath
A rust crate for simple path (like PATH and LD_PATH) use.

Other crates can create a simpath that loads from the path defined as an environment variable using:

```
let path = simppath::new("LD_PATH");
let file = path.find("filename");
```

##Methods
Methods exist to find a file on the path as a PathBuf or as a URL with the "file:" scheme.

The path can be extended with additional directories to search in, but this won't affect the value of the environment 
variable that was loaded initially.