---
sidebar_position: 3
---

# Your First Deploy
Now that we have Hydro Deploy installed, let's deploy our first app. We'll start with a simple app that echoes packets.

First, we need to write the Hydroflow application, which will intergrate with Hydro Deploy to initialize connections to other services. We can create a project using the Cargo template:

```bash
# if you don't already have cargo-generate installed
#shell-command-next-line
cargo install --locked cargo-generate

#shell-command-next-line
cargo generate gh:hydro-project/hydroflow template/hydroflow
```

`cd` into the generated folder, ensure the correct nightly version of rust is installed, and test the generated project:
```bash
#shell-command-next-line
cd <my-project>
#shell-command-next-line
rustup update
#shell-command-next-line
cargo test
```

We'll need to add an additional dependency for `hydroflow_deploy_integration` to our `Cargo.toml`:

```toml
[dependencies]
# ...
hydroflow_deploy_integration = "0.1.1"
```

Let's open up `src/main.rs` in the generated project and write a new `main` function that initializes Hydro Deploy:

```rust
#[dfir_rs::main]
async fn main() {
    let ports = dfir_rs::util::deploy::init().await;
}
```

This ports value gives us access to any network ports we define in our Hydro Deploy configuration. Let's create that next, in a `echo.hydro.py` file. A Hydro Deploy script consists of an asynchronous `main` function which can interactively create and destroy services. We start by creating a new deployment, which tracks the active resources and services:

```python
import hydro

async def main(args):
    deployment = hydro.Deployment()
```

Next, we pick out the host we want to run the service on (localhost for now), and create a pair of services on that host.

```python
    host = deployment.Localhost()
    echo_service_1 = deployment.HydroflowCrate(
        src=".",
        on=host
    )

    echo_service_2 = deployment.HydroflowCrate(
        src=".",
        on=host
    )
```

Now, we need to wire up the ports. Hydro Deploy uses _named ports_, which can then be loaded in our Hydroflow logic. In our example, each echo service will have an "input" and "output" port. We can wire them up using the `send_to` method:

```python
    echo_service_1.ports.output.send_to(echo_service_2.ports.input)
    echo_service_2.ports.output.send_to(echo_service_1.ports.input)
```

Returning briefly to our Hydroflow code, we can then load these ports and use them to send and receive packets:

```rust
use hydroflow_deploy_integration::ConnectedDirect;
use dfir_rs::dfir_syntax;

#[dfir_rs::main]
async fn main() {
    let ports = dfir_rs::util::deploy::init().await;

    let input_recv = ports
        .port("input")
        // connect to the port with a single recipient
        .connect::<ConnectedDirect>()
        .await
        .into_source();

    let output_send = ports
        .port("output")
        .connect::<ConnectedDirect>()
        .await
        .into_sink();

    dfir_rs::util::deploy::launch_flow(dfir_syntax! {
        source_iter(["hello".to_string()]) -> dest_sink(output_send);
        input = source_stream(input_recv) -> tee();
        input -> dest_sink(output_send);
        input -> for_each(|s| println!("{}", std::str::from_utf8(s).unwrap()));
    }).await;
}
```

Finally, we can return to `echo.hydro.py` to launch the application. First, `deploy` compiles the binaries, initializes hosts, and sets up the networking topology. Then, we use `start` to launch the services:

```python
    await deployment.deploy()
    await deployment.start()
```

To run the deployment, we can use the `hydro deploy` command:

```bash
#shell-command-next-line
hydro deploy echo.hydro.py
```

And if all goes well, we should see the packets being echoed back and forth between the two services!
