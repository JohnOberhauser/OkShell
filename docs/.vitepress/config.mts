import { defineConfig } from 'vitepress'

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: "OkShell",
  description: "A customizable desktop shell for Hyprland",
  themeConfig: {
    // https://vitepress.dev/reference/default-theme-config
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
      },
      {
        text: 'Dev info',
        items: [
          { text: 'Markdown Examples', link: '/markdown-examples' },
          { text: 'Runtime API Examples', link: '/api-examples' }
        ]
      }
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/JohnOberhauser/OkShell' }
    ]
  }
})
