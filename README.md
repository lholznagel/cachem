# Cachem WIP

A "database" without batteries included.
The idea is that the needed models and there necessary parsing to and from
bytes are implemented.
Thats why this repo only includes basic functions and wrappers for
a handfull of datatypes.
These wrappers can be used to build more complex structures that represent
the actual data.

Besides that, the "database" has no user authentication, query language
or something similar that most databases have.
This "database" can be more considered a thin wrapper for data that is
accessible over the network.

Instead of having tables or collections, this "database" has caches.
These caches are only in memory.
Only when configured these caches are saved to disk when the database gets
a SIGINT signal.
On startup those files are then loaded.

## Disadvantages

- Because everything is kept in memory, the memory allocation amount can be high, of course depending on the amount of data that is stored
- The caches are only saved to disk when a SIGINT (CTRL+C) is received, if the server crashes for some reason, there will be data loss
- There is no user authentication, query language or filtering besides filtering by id
- Initial manual work, but parts of it is supported by using proc macros

## Advantages

- Small protocol overhead, in most cases it is either 2 bytes or 6 bytes overhead
  - 1 byte: Action that should be performed (fetch, delete, update, insertt)
  - 2 byte: Cache that should be used
  - 3 byte to 6 byte: If an vec is transmited, this contains the number of elements in the vec
  - When sending strings the overhead gets higher, every string is 0 byte terminated
    - If you send a vec containing 100 strings, the overhead will be 106 bytes
    - The recommendation is to have a cache that only handles resolving ids to strings
- Fast
- The "database" is specific for one project, depending of what is stored, the model can be designed to be as efficient as possible

## Usage

Currently I do not plan to make this crate available over at crates.io.
Reason being that I don´t want to spam crates.io with crates that are more or
less only for personal use.
If there is interest in publishing them I will consider it.

``` toml
# Cargo.toml

async-trait = "0.1"
cachem_utils = { git = "https://github.com/lholznagel/cachem.git", rev = "INSERT_LATEST_GIT_REV", features = ["derive"] }
tokio = { version = "1.1", features = ["full"] }
```

The `cachem_utils` crate contains all needed traits and functions.
Most of the trait can be implemented using proc-macros.
For the proc macros, the feature `derive` must be added.

By adding `with-uuid` the support for uuid´s is enabled.
This implements the parser trait for `Uuid`.

### Example

See [main.rs](./example/src/main.rs)

The following code is an example for a minimal server example.

``` rust
use async_trait::async_trait;
use cachem_utils::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Debug, Parse)]
pub struct SampleEntry {
    pub id:    u32,
    pub val_1: u32,
    pub val_2: bool,
    pub val_3: u64,
}

#[derive(Debug, Parse)]
pub struct FetchSampleEntryById(pub u32);

#[derive(Default)]
pub struct SampleCache(RwLock<HashMap<u32, SampleEntry>>);

#[async_trait]
impl Fetch<FetchSampleEntryById> for SampleCache {
    type Error = ();
    type ReturnType = SampleEntry;

    async fn fetch(&self, input: FetchSampleEntryById) -> Result<Self::ReturnType, ()> {
        if let Some(x) = self.0.read().await.get(&input.0) {
            Ok(x.clone())
        } else {
            Err(())
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sample_cache = Arc::new(SampleCache::default());

    #[derive(Debug)]
    enum Actions {
        Fetch
    }
    impl From<u8> for Actions {
        fn from(x: u8) -> Self {
            match x {
                0 => Actions::Fetch,
                _ => panic!("Invalid action")
            }
        }
    }

    #[derive(Debug)]
    enum Caches {
        Sample
    }
    impl From<u8> for Caches {
        fn from(x: u8) -> Self {
            match x {
                0 => Caches::Sample,
                _ => panic!("Invalid action")
            }
        }
    }

    cachem! {
        "0.0.0.0:9999",

        let sample_copy = sample_cache.clone();

        (Actions::Fetch, Caches::Sample) => (sample_copy, fetch, FetchSampleEntryById),
    }
}
```

There are some restrictions
- There must exist an enum called `Actions` and `Caches`
  - Both must implement `Debug`
  - Both must implement `From<u8>`
- From every cache implementation there must be a copy `let sample_copy = sample_cache.clone();`
  - It is recommended to init them as `let sample_cache = Arc::new(SampleCache::default());`
  - Every Cache should either use a `RwLock` or `Mutex`
- After that, all "routes" must be defined
  - `(Actions::Fetch, Caches::Sample) => (cache_copy, function_name, request_type),`
    - Please see `Example route` for more information

### Example route

The library provides traits for the following actions: `Fetch`, `Lookup`, `Insert`, `Update` and `Delete`.
You can implement your own traits and call them from a route.

``` rust
#[async_trait]
pub trait MyCustomTrait<T: Parse> {
    async fn call_me(&self, data: T) -> Result<SampleEntry, ()>;
}
```

When implementing your own trait it is important that it is an async trait and that it takes `&self` and another parameter.
That parameter is used to inject the request into the function.
Besides that a result must be returned.
The data parameter and the result must both implement `Parse`.

The route configuration in the macro looks something like this:

> `(Actions::Fetch, Caches::Sample) => (cache_copy, function_name, request_type),`.

- The first parameter `cache_copy` is the actual cache that should be used
- The `function_name` can be anything that the cache implements, it is recommendet to use a trait, so that the function can be implemented mutliple time with data types.
- The `request_type` is the model that represents the model that comes from the tcp socket.

## The `Parse` trait

``` rust
use cachem_utils::Parse;

#[derive(Parse)]
pub struct MyCacheEntry {
  my_val_1: u32,
  my_val_2: String,
}
```

The `Parse` trait includes all functions that are needed in order to read and
write the model into and from bytes.
By using the proc-macro the trait is automatically implemeted.

The code above would boil down to this code:

``` rust
use async_trait::async_trait;
use cachem_utils::Parse;
use tokio::io::{AsyncBufRead, AsyncRead, AsyncWrite};

pub struct MyCacheEntry {
  my_val_1: u32,
  my_val_2: String,
}

#[async_trait]
impl Parse for MyCacheEntry {
  async fn read<B>(
        buf: &mut B
    ) -> Result<Self, CachemError>
    where
        B: AsyncBufRead + AsyncRead + Send + Unpin {

    Ok(MyCacheEntry {
      my_val_1: u32::read(buf).await?,
      my_val_2: String::read(buf).await?
    })
  }

  async fn write<B>(
      &self,
      buf: &mut B
  ) -> Result<(), CachemError>
  where
      B: AsyncWrite + Send + Unpin {

    self.my_val_1.write(buf).await?;
    self.my_val_2.write(buf).await?;
    Ok(())
  }
}
```

The `Parse` trait is also implemented for the datatypes `u32`, `u64`, `u128`,
`f32`, `f64`, `String` and `bool`.
With the feature `with-uuid` the type `Uuid` also has the `Parse` trait implemented.
With that models can be easily designed.

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Serde by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
</sub>
