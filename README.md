# lumos
My lighting control engine for my place, written in Rust.

*Lumos* is the spell in the Harry Potter world used to create light at the end of a wand. 
In my apartment, *lumos* is the spell that turns IPC-based commands into lighting commands 
for various endpoints: right now Nanoleaf and Hue. Lumos sets up a Unix domain socket to 
listen for commands containing an identifier for a button and based on the identifier can 
execute programming.

## Improvements for the Future
* Serialized form (like JSON) for the programming so that it isn't hardcoded into the engine
* Programming creation workflow for associating buttons with programming at runtime
* Add integrations for Lutron Caseta
* Clean up the binary's architecture into a modular design split over various pieces
* Define a spec for the command protocol
  * Parse the spec and create protocol serde library
* Make an async-based Hue crate
