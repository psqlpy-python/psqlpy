import { defineUserConfig } from "vuepress";
import { hopeTheme } from "vuepress-theme-hope";
import sidebar from "./sidebar.js";
import { markdownTabPlugin } from '@vuepress/plugin-markdown-tab'

export default defineUserConfig({
  lang: "en-US",
  title: "PSQLPy",
  description: "PSQLPy Documentation",

  // bundler: viteBundler(),

  theme: hopeTheme({
    repo: "psqlpy-python/psqlpy",

    repoLabel: "GitHub",

    repoDisplay: true,

    sidebar,

    hostname: "https://psqlpy-python.github.io/",

    markdown: {
      tabs: true,
      mermaid: true,
      chartjs: true,
    },

    plugins: {
      readingTime: false,
      copyCode: {
        showInMobile: true,
      },

      slimsearch: {
        indexContent: true,
      },

      sitemap: {
        changefreq: "daily",
        sitemapFilename: "sitemap.xml",
      },

    },
  }),
});
