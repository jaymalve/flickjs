import { defineConfig } from "vitepress";

export default defineConfig({
  title: "Flick",
  description: "A tiny reactive JS framework from the future",
  base: "/flick/",

  // Force dark mode only - no light mode toggle
  appearance: "force-dark",

  head: [
    [
      "link",
      { rel: "icon", type: "image/svg+xml", href: "/flick/favicon.svg" },
    ],
  ],

  themeConfig: {
    logo: "/logo.svg",

    nav: [
      { text: "Guide", link: "/guide/" },
      { text: "API", link: "/api/" },
      { text: "Examples", link: "/examples/" },
    ],

    sidebar: {
      "/": [
        {
          text: "Introduction",
          items: [
            { text: "Getting Started", link: "/guide/" },
            { text: "Installation", link: "/guide/installation" },
          ],
        },
        {
          text: "Core Concepts",
          items: [
            { text: "Signals", link: "/guide/signals" },
            { text: "Components", link: "/guide/components" },
            { text: "Effects", link: "/guide/effects" },
          ],
        },
        {
          text: "Styling",
          items: [{ text: "CSS & Tailwind", link: "/guide/styling" }],
        },
        {
          text: "Advanced",
          items: [{ text: "TypeScript", link: "/guide/typescript" }],
        },
      ],
      "/guide/": [
        {
          text: "Introduction",
          items: [
            { text: "Getting Started", link: "/guide/" },
            { text: "Installation", link: "/guide/installation" },
          ],
        },
        {
          text: "Core Concepts",
          items: [
            { text: "Signals", link: "/guide/signals" },
            { text: "Components", link: "/guide/components" },
            { text: "Effects", link: "/guide/effects" },
          ],
        },
        {
          text: "Styling",
          items: [{ text: "CSS & Tailwind", link: "/guide/styling" }],
        },
        {
          text: "Advanced",
          items: [{ text: "TypeScript", link: "/guide/typescript" }],
        },
      ],
      "/api/": [
        {
          text: "API Reference",
          items: [
            { text: "Overview", link: "/api/" },
            { text: "Runtime", link: "/api/runtime" },
            { text: "Router", link: "/api/router" },
            { text: "Suspense", link: "/api/suspense" },
          ],
        },
      ],
      "/examples/": [
        {
          text: "Examples",
          items: [{ text: "Project Structure", link: "/examples/" }],
        },
      ],
    },

    socialLinks: [
      { icon: "github", link: "https://github.com/jaymalave/flick" },
    ],

    footer: {
      message: "Released under the MIT License.",
      copyright: "Copyright © 2024 Jay Malave",
    },

    search: {
      provider: "local",
    },
  },
});
