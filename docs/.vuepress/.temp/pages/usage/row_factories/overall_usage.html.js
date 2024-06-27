import comp from "/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/usage/row_factories/overall_usage.html.vue"
const data = JSON.parse("{\"path\":\"/usage/row_factories/overall_usage.html\",\"title\":\"Generic usage of row_factory\",\"lang\":\"en-US\",\"frontmatter\":{\"title\":\"Generic usage of row_factory\",\"description\":\"row_factory must be used when you want to process result from Database in a custom way and return something different from dictionary. row_factory requires a function that accep...\",\"head\":[[\"meta\",{\"property\":\"og:url\",\"content\":\"https://qaspen-python.github.io/usage/row_factories/overall_usage.html\"}],[\"meta\",{\"property\":\"og:site_name\",\"content\":\"PSQLPy\"}],[\"meta\",{\"property\":\"og:title\",\"content\":\"Generic usage of row_factory\"}],[\"meta\",{\"property\":\"og:description\",\"content\":\"row_factory must be used when you want to process result from Database in a custom way and return something different from dictionary. row_factory requires a function that accep...\"}],[\"meta\",{\"property\":\"og:type\",\"content\":\"article\"}],[\"meta\",{\"property\":\"og:locale\",\"content\":\"en-US\"}],[\"script\",{\"type\":\"application/ld+json\"},\"{\\\"@context\\\":\\\"https://schema.org\\\",\\\"@type\\\":\\\"Article\\\",\\\"headline\\\":\\\"Generic usage of row_factory\\\",\\\"image\\\":[\\\"\\\"],\\\"dateModified\\\":null,\\\"author\\\":[]}\"]]},\"headers\":[{\"level\":3,\"title\":\"Example:\",\"slug\":\"example\",\"link\":\"#example\",\"children\":[]}],\"filePathRelative\":\"usage/row_factories/overall_usage.md\",\"autoDesc\":true,\"excerpt\":\"<p><code>row_factory</code> must be used when you want to process result from Database in a custom way and return something different from dictionary.</p>\\n<p><code>row_factory</code> requires a function that accepts parameter <code>Dict[str, typing.Any]</code> and can return anything you want.</p>\\n<div class=\\\"hint-container tip\\\">\\n<p class=\\\"hint-container-title\\\">Tips</p>\\n<p><code>row_factory</code> can be a function or a class with <code>__call__</code> method which returns target converted instance.</p>\\n</div>\"}")
export { comp, data }

if (import.meta.webpackHot) {
  import.meta.webpackHot.accept()
  if (__VUE_HMR_RUNTIME__.updatePageData) {
    __VUE_HMR_RUNTIME__.updatePageData(data)
  }
}

if (import.meta.hot) {
  import.meta.hot.accept(({ data }) => {
    __VUE_HMR_RUNTIME__.updatePageData(data)
  })
}
