# Icons

`ferrostage.png` is the source art: a chibi red panda. Fur uses DESIGN.md primary `#A34F35`, the muzzle uses canvas `#FAF9F5`, and the eye mask uses ink `#141413`.

`ferrostage.ico` contains 16, 32, 48, and 256 pixel images. `crates/ferrostage-app/build.rs` embeds that file as Windows `ICON` resource ID 1. The Windows shell reads resource ID 1 for the window-class icon.
