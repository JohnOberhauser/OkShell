# Matugen

[Matugen](https://github.com/InioX/matugen) is a material you and base16 color generation tool.
Its main function is to help users generate color palettes from wallpapers and apply those colors to all themable programs.

All non-default, built-in theming in OkShell happens through Matugen.

You can change your theme in `Settings` -> `Theme` -> `Color Scheme`.
If you choose `Default`, Matugen will not be used.
If you choose `Wallpaper`, the theme will change based on the wallpaper you have selected.
If you choose a static theme, Matugen will apply that static theme to all your programs configured with Matugen.

::: tip
If you want Matugen to automatically change the themes of other programs on your computer, you will need to configure Matugen yourself.
See the [documentation](https://iniox.github.io/#matugen) to get started.
:::

## OkShell Expressions

Some non-color values in theme settings are sent to Matugen so you can keep your app themes in sync:

```
{{ okshell.font.primary }}
{{ okshell.font.secondary }}
{{ okshell.font.tertiary }}
{{ okshell.sizing.radius_widget }}
{{ okshell.sizing.radius_window }}
{{ okshell.sizing.border_width }}
{{ okshell.opacity }}
```

## What if I don't want to theme with Matugen

You don't have to.  Set your color scheme to `Default` and check out the next section about style sheets.