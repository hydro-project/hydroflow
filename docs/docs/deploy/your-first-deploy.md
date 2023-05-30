---
sidebar_position: 2
---

# Your First Deploy
Now that we have Hydro Deploy installed, let's deploy our first app. We'll start with a simple app that echos packets back to us.

## Connecting Hydroflow to Hydro Deploy
First, we need to write the Hydroflow application, which will intergrate with Hydro Deploy to initialize connections to other services. We can create a project using the Cargo template:

```bash
# if you don't already have cargo-generate installed
#shell-command-next-line
cargo install --locked cargo-generate

#shell-command-next-line
cargo generate hydro-project/hydroflow-template
```

Let's open up `src/main.rs` in the generated project and write a new `main` function that initializes Hydro Deploy:

```rust
#[hydroflow::main]
async fn main() {
    let mut ports = hydroflow::util::cli::init().await;
}
```

This ports value gives us access to any network ports we define in our Hydro Deploy configuration. Create that next, in a `echo.hydro.py` file:

```python
async def main(args):
  