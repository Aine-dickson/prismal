# Examples

Complete programs written the way an author would write them: a model, a lab and a narrated lesson, with the cases that check them. Every program here is compiled and its cases run by `cargo test` (`crates/prismal-web/tests/examples.rs`). Each page starts with its goal and how it is built.

| Example | Shows |
|---|---|
| [Which angle throws farthest?](projectile.md) | a projectile thrown again on request; a lab with slider ends, live texts, a growing plot and the range formula; a lesson of three throws and an explore beat; video export |
| [Why a² + b² = c²](pythagoras.md) | a figure with no state, drawn from its parameters; two arrangements side by side with drag handles; a lesson with camera moves, triangles morphed into place and squares revealed |
| [Where does a chain break?](chain.md) | objects and an undirected relation: beads joined by links that snap; clicks on members, a weight as a member payload; a lesson that requests for a member, intervenes, resets and hands over a slider |
| [Few molecules, many molecules](reaction.md) | a reversible reaction as random events beside its rate equation; counts drawn as staircases; a histogram of 200 runs that follows the learner's number of molecules; `restart` and `reseed` in a lab and a lesson |

To open one in the web player, paste its program blocks into the Source tab. To export a lesson as a video:

```sh
cargo run --release -p prismal-media -- examples/projectile.md ThrowLesson throw.mp4
```
