import { defineClientConfig } from "vuepress/client";
import { useHintContainers } from "/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/node_modules/.pnpm/vuepress-plugin-md-enhance@2.0.0-rc.36_markdown-it@14.1.0_mermaid@10.9.1_sass-loader@14.2.1_vuepress@2.0.0-rc.9/node_modules/vuepress-plugin-md-enhance/lib/client/composables/useHintContainers.js";
import "/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/node_modules/.pnpm/vuepress-plugin-md-enhance@2.0.0-rc.36_markdown-it@14.1.0_mermaid@10.9.1_sass-loader@14.2.1_vuepress@2.0.0-rc.9/node_modules/vuepress-plugin-md-enhance/lib/client/styles/hint/index.scss";
import Mermaid from "/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/node_modules/.pnpm/vuepress-plugin-md-enhance@2.0.0-rc.36_markdown-it@14.1.0_mermaid@10.9.1_sass-loader@14.2.1_vuepress@2.0.0-rc.9/node_modules/vuepress-plugin-md-enhance/lib/client/components/Mermaid.js";
import { injectMermaidConfig } from "/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/node_modules/.pnpm/vuepress-plugin-md-enhance@2.0.0-rc.36_markdown-it@14.1.0_mermaid@10.9.1_sass-loader@14.2.1_vuepress@2.0.0-rc.9/node_modules/vuepress-plugin-md-enhance/lib/client//index.js";
import Tabs from "/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/node_modules/.pnpm/vuepress-plugin-md-enhance@2.0.0-rc.36_markdown-it@14.1.0_mermaid@10.9.1_sass-loader@14.2.1_vuepress@2.0.0-rc.9/node_modules/vuepress-plugin-md-enhance/lib/client/components/Tabs.js";

export default defineClientConfig({
  enhance: ({ app }) => {
    injectMermaidConfig(app);
    app.component("Mermaid", Mermaid);
    app.component("Tabs", Tabs);
  },
  setup: () => {
useHintContainers();
  }
});
