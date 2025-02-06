# JS-Suboptimizer

Currently just a toy that will look for a marker, `// split`, and take the subsequent Javascript function and move it out
to a module file.  A new, updated html is created with an import statement added the references all moved methods along with
a module file containing the moved functions.

## Usage
```
cargo run --release --bin js-suboptimizer <path to html file>  optionally (--package-name <package name> )
```

### Examples
```bash 
cargo run --release --bin js-suboptimizer simple.html
```

This a suboptimzed version of the file `./simple/simple.html` that was created by the above command along with a module
file `./simmple/simple-module.js` that contains all functions moved from the original `./simple`.

```bash 
cargo run --release --bin js-suboptimizer simple.html --package-name my-package
```

Similar to the above command, however the output directory will be `./my-package` instead of `./simple`.
