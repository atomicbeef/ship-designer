# Space ship designer game
This is an incomplete voxel-based spaceship building game that I was working on, utilizing the Bevy engine.

## Features
- Multiplayer (not fully synchronized) with a server and client architecture
- Building: place and delete objects composed of 10 centimeter wide voxels
- Missiles with voxel destruction
- Homemade serialization code (plus a derive macro to reduce boilerplate) for networking
- Modular project design consisting of multiple crates
