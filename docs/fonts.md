# Fonts

You can change the font in `Settings` -> `Theme` -> `Fonts`.

There are three font options to change.  Primary, secondary, and tertiary.
Primary is the only font used in OkShell, unless you add a style sheet that uses the secondary or tertiary font variables.

When you change a font, the current Matugen theme will be reapplied.
Custom variables are available in Matugen to access the font values.

```
{{ okshell.font.primary }}
{{ okshell.font.secondary }}
{{ okshell.font.tertiary }}
```

You can use these to sync the font of other programs, like your terminal, IDE, or GTK.

::: info NOTE
Font will not be applied through Matugen when using the Default theme.
:::