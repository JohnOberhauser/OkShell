---
# https://vitepress.dev/reference/default-theme-home-page
layout: home

hero:
  name: "OkShell"
  text: "A customizable desktop shell for Hyprland"
  tagline: It's okay.
  image: /assets/logo.svg
  actions:
    - theme: brand
      text: Get Started
      link: /installation
    - theme: alt
      text: Github
      link: https://github.com/JohnOberhauser/OkShell

features:
  - title: Everything you need
    details: Widgets, notifications, lock screen, polkit agent, wallpaper management, screenshot tool, and more included!
  - title: Customize bars and menus
    details: Place your bar(s) on any edge of the screen and customize the layout of your menus
  - title: Built-in Matugen support
    details: Have your theme change dynamically to match your wallpaper, or choose a built-in static theme
---

<script setup>
import ScreenshotSlideshow from './components/ScreenshotSlideshow.vue'

const shots = [
  { src: '/assets/screenshot1.png',        alt: 'OkShell desktop' },
  { src: '/assets/screenshot2.png', alt: 'Calendar and weather' },
  { src: '/assets/screenshot3.png',      alt: 'Wallpaper picker and theme' },
  { src: '/assets/screenshot4.png',      alt: 'Quick settings menu' },
  { src: '/assets/screenshot5.png',      alt: 'Settings app' },
]
</script>

<ScreenshotSlideshow :images="shots" :interval="4000" />
