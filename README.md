# ttbl-rs

Tool for generating Truth Tables.

## Building

1. Make sure you have a rust toolchain installed (cargo and a rust compiler)
   
   ```sh
   rustc --version
   cargo --version
   ```
2. Clone the repository:

   ```sh
   git clone https://github.com/enix403/ttbl-rs.git
   ```

3. `cd` into the root folder and run:

   ```sh
   cargo build --release
   # This above command might take some time to complete
   
   # Now start the program
   ./target/release/ttbl
   ```
   
## Usage

Just enter valid boolean expression and press enter
#### Examples

- `p and q`
- `p & (p or q)`
- `!p1 | p2`
- `a and FALSE` (here `FALSE` is the literal _"false"_ and not a variable)


Wrap a valid subexpression in braces (`{` and `}`) to generate a separate dedicated column for the subexpression.

- `{p or q} and !(p and {!q})`
- `!{a & {b or c}}`

