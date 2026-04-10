import { defineConfig } from 'vitepress'

export default defineConfig({
  title: "OkShell",
  base: "/OkShell/",
  description: "A customizable desktop shell for Hyprland",
  head: [
    ['link', { rel: 'icon', href: '/OkShell/assets/logo.svg' }]
  ],
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
          { text: 'Features', link: '/features' },
          { text: 'Hyprland screen share', link: '/screensharing' }
        ]
      },
      {
        text: 'Configuration',
        items: [
          { text: 'Settings', link: '/settings' },
          { text: 'Schema', link: '/schema' }
        ]
      },
      {
        text: 'Theming',
        items: [
          { text: 'Matugen', link: '/matugen' },
          { text: 'Style sheets', link: '/style_sheets' },
          { text: 'Icons', link: '/icons' },
          { text: 'Fonts', link: '/fonts' }
        ]
      }
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/JohnOberhauser/OkShell' }
    ]
  }
})
