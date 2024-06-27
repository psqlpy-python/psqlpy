export const themeData = JSON.parse("{\"encrypt\":{},\"repo\":\"qaspen-python/psqlpy\",\"repoLabel\":\"GitHub\",\"repoDisplay\":true,\"locales\":{\"/\":{\"lang\":\"en-US\",\"navbarLocales\":{\"langName\":\"English\",\"selectLangAriaLabel\":\"Select language\"},\"metaLocales\":{\"author\":\"Author\",\"date\":\"Writing Date\",\"origin\":\"Original\",\"views\":\"Page views\",\"category\":\"Category\",\"tag\":\"Tag\",\"readingTime\":\"Reading Time\",\"words\":\"Words\",\"toc\":\"On This Page\",\"prev\":\"Prev\",\"next\":\"Next\",\"lastUpdated\":\"Last update\",\"contributors\":\"Contributors\",\"editLink\":\"Edit this page\",\"print\":\"Print\"},\"outlookLocales\":{\"themeColor\":\"Theme Color\",\"darkmode\":\"Theme Mode\",\"fullscreen\":\"Full Screen\"},\"routeLocales\":{\"skipToContent\":\"Skip to main content\",\"notFoundTitle\":\"Page not found\",\"notFoundMsg\":[\"There’s nothing here.\",\"How did we get here?\",\"That’s a Four-Oh-Four.\",\"Looks like we've got some broken links.\"],\"back\":\"Go back\",\"home\":\"Take me home\",\"openInNewWindow\":\"Open in new window\"},\"sidebar\":{\"/\":[\"\",{\"text\":\"The PSQLPy\",\"prefix\":\"introduction/\",\"collapsible\":true,\"children\":[\"introduction\",\"lets_start\"]},{\"text\":\"Components Overview\",\"prefix\":\"components/\",\"collapsible\":true,\"children\":[\"components_overview\",\"connection_pool\",\"connection_pool_builder\",\"connection\",\"transaction\",\"cursor\",\"results\",\"exceptions\"]},{\"text\":\"Usage\",\"prefix\":\"usage/\",\"collapsible\":true,\"children\":[{\"text\":\"Types\",\"prefix\":\"types/\",\"collapsible\":true,\"children\":[\"supported_types\",\"extra_types\",\"advanced_type_usage\"]},{\"text\":\"Frameworks Usage\",\"prefix\":\"frameworks/\",\"collapsible\":true,\"children\":[\"aiohttp\",\"fastapi\",\"litestar\",\"blacksheep\",\"robyn\"]},{\"text\":\"Row Factories Usage\",\"prefix\":\"row_factories/\",\"collapsible\":true,\"children\":[\"row_factories\",\"overall_usage\",\"predefined_row_factories\"]}]},{\"text\":\"Contribution guide\",\"prefix\":\"/contribution_guide\",\"link\":\"/contribute.md\"}]}}}}")

if (import.meta.webpackHot) {
  import.meta.webpackHot.accept()
  if (__VUE_HMR_RUNTIME__.updateThemeData) {
    __VUE_HMR_RUNTIME__.updateThemeData(themeData)
  }
}

if (import.meta.hot) {
  import.meta.hot.accept(({ themeData }) => {
    __VUE_HMR_RUNTIME__.updateThemeData(themeData)
  })
}
