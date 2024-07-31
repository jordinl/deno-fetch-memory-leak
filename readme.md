# Deno fetch memory leak

Show memory leak in deno fetch. It's better to run this example on a server where concurrency can be set to a high 
number, such as 100. 

### Running the example

Make sure you have deno installed first.

Clone this repo and execute:

```
./run
```

#### Concurrency

By default the concurrency is set to 10. It can be changed by setting the `CONCURRENCY` environment variable.

```
CONCURRENCY=100 ./run
```

## Rust example

There is a rust example in the `rust` folder. It uses the `reqwest` library to make the requests.

### Requirements

- Rust
- Cargo
- build-essential (`sudo apt install build-essential`)

To compile it run:

```angular
rust/compile
```

To run it:

```
rust/run
```

`rust/run` also accepts the `CONCURRENCY` environment variable.
