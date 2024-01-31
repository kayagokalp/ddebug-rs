# ddebug-rs
A differential debugger for rust source code. The tools tries to minimize input rust code into it's minimally reproducable version throwing the same error as before. Rather than trying to remove characther by characther the tool relies on AST.

## Example
For the example project (located in the repo's test folder as well) containing the following code piece, `ddebug-rs` able to minimize the code into its more compact form.

```rust
fn main() {
    let b = 0;
    let a = 0;
    let c = 0;
    b = 10;
}
```

```sh
> ddebug-rs
Minimized the code into:
fn main() {
    let b = 0;
    b = 10;
}
```

## High level overview
Given a path, or the current path the cli is invoked, `cargo build` is called with some filters to understand what type of errors the given input rust code creates first. After deciding the target error and the file it is sourced from, the tool parses that file into an AST. AST nodes are traversed and by removing and retrying the build process actual required set of nodes are determined.
One thing to note here is that `ddebug-rs` tries to remove large nodes first to mark entire subgraphs of AST unnecssary. So it should be a faster than a standard delta debugging tool trying to rely on string manipulation.

## Notice
 1. `ddebug-rs` is written in a weekend and mostly stands as a POC, lots of unnecessary `clone`'s are in the code-base due to my laziness. So by performance standards it is far from optimal. 
 2. `ddebug-rs` does not cover the entire rust syntax space, it is covering a very little subset of valid syntax which is something I will be working on from time to time.

As mentioned above the project is by no means optimal. Happy to take in any contributions, especially around the 2 points given above

## Contributions
Highly appreciated.
