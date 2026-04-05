# Style Sheets

OkShell uses GTK4, which means you can change the style with css.

## Loading a style sheet

Place the css file you want to load in `~/.config/okshell/styles/`.
Then go to `Settings` -> `Theme` -> `Custom CSS` to select the style sheet you want to load.
You can modify your style sheet while using it and see live updates.

## Changing global colors and sizing

There are a handful of css variable you can change in `:root` that will affect the entire shell.
You can see the variables [here](https://github.com/JohnOberhauser/OkShell/tree/main/crates/okshell-style/scss/01-tokens).

## Finding CSS classes

Use `okshellctl inspect` to open the GTK4 inspector.
