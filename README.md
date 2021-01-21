# Cachem

A "database" without batteries included.
The idea is that the needed models and there necessary parsing to and from
bytes are implemented.
Thats why this "database" only includes basic functions and wrapper for
a handfull of datatypes.
These wrappers can be used to build more complex structures that represent
the actual data.

Besides that, the "database" has no user authentication, query language
or something similar that most databases have.
This "database" can be more considered a thin wrapper for data that is
accessible over the network.

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