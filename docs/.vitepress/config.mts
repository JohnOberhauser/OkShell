import { defineConfig } from 'vitepress'

export default defineConfig({
  title: "OkShell",
  base: "/OkShell/",
  description: "A customizable desktop shell for Hyprland",
  themeConfig: {
    nav: [
      { text: 'Home', link: '/' },
    ],

    sidebar: [
      {
        text: 'Getting Started',
        items: [
          { text: 'Installation', link: '/installation' },
          { text: 'Usage', link: '/usage' },
          { text: 'Features', link: '/features' }
        ]
      },
      {
        text: 'Configuration',
        items: [
          { text: 'Settings UI', link: '/settings' },
          { text: 'Schema', link: '/schema' }
        ]
      },
      {
        text: 'Theming',
        items: [
          { text: 'Matugen', link: '/matugen' },
          { text: 'Style sheets', link: '/style_sheets' },
          { text: 'Icons', link: '/icons' }
        ]
      }
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/JohnOberhauser/OkShell' }
    ]
  }
})
