import { sidebar } from "vuepress-theme-hope";

export default sidebar({
  "/": [
    "",
    {
      text: "The PSQLPy",
      prefix: "introduction/",
      collapsible: true,
      children: [
        "introduction",
        "lets_start",
      ],
    },
    {
      text: "Components Overview",
      prefix: "components/",
      collapsible: true,
      children: [
        "components_overview",
        "connection_pool",
        "connection_pool_builder",
        "connection",
        "transaction",
        "cursor",
        "listener",
        "results",
        "exceptions",
      ],
    },
    {
      text: "Usage",
      prefix: "usage/",
      collapsible: true,
      children: [
        "parameters",
        {
          text: "Types",
          prefix: "types/",
          collapsible: true,
          children: [
            "supported_types",
            "array_types",
            "extra_types",
            "advanced_type_usage",
          ]
        },
        {
          text: "Frameworks Usage",
          prefix: "frameworks/",
          collapsible: true,
          children: [
            "aiohttp",
            "fastapi",
            "litestar",
            "blacksheep",
            "robyn",
          ]
        },
        {
          text: "Row Factories Usage",
          prefix: "row_factories/",
          collapsible: true,
          children: [
            "row_factories",
            "predefined_row_factories",
          ]
        },
      ],
    },
    {
      text: "Integrations",
      prefix: "/integrations",
      collapsible: true,
      children: [
        "taskiq",
        "opentelemetry",
      ],
    },
    {
      text: "Contribution guide",
      prefix: "/contribution_guide",
      link: "/contribute.md"
    },
    {
      text: "Benchmarks",
      prefix: "/benchmarks",
      link: "/benchmarks.md"
    },
    {
      text: "FAQ",
      prefix: "/faq",
      link: "/faq.md"
    },
  ],
});
