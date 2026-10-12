# Examples

Complete programs written the way an author would write them: a model, a lab and a narrated lesson, with the cases that check them. Every program here is compiled and its cases run by `cargo test` (`crates/prismal-web/tests/examples.rs`). Each page starts with its goal and how it is built.

| Example | Shows |
|---|---|
| [Which angle throws farthest?](projectile.md) | a projectile thrown again on request; a lab with slider ends, live texts, a growing plot and the range formula; a lesson of three throws and an explore beat; video export |

To open one in the web player, paste its program blocks into the Source tab. To export a lesson as a video:

```sh
cargo run --release -p prismal-media -- examples/projectile.md ThrowLesson throw.mp4
```
