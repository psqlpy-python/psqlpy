import comp from "/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/introduction/components_overview.html.vue"
const data = JSON.parse("{\"path\":\"/introduction/components_overview.html\",\"title\":\"Components Overview\",\"lang\":\"en-US\",\"frontmatter\":{\"title\":\"Components Overview\",\"description\":\"Components Connection pool: holds connections in itself and give them when requested. Connection: represents single database connection, can be retrieved from Connection pool. T...\",\"head\":[[\"meta\",{\"property\":\"og:url\",\"content\":\"https://qaspen-python.github.io/introduction/components_overview.html\"}],[\"meta\",{\"property\":\"og:site_name\",\"content\":\"PSQLPy\"}],[\"meta\",{\"property\":\"og:title\",\"content\":\"Components Overview\"}],[\"meta\",{\"property\":\"og:description\",\"content\":\"Components Connection pool: holds connections in itself and give them when requested. Connection: represents single database connection, can be retrieved from Connection pool. T...\"}],[\"meta\",{\"property\":\"og:type\",\"content\":\"article\"}],[\"meta\",{\"property\":\"og:locale\",\"content\":\"en-US\"}],[\"script\",{\"type\":\"application/ld+json\"},\"{\\\"@context\\\":\\\"https://schema.org\\\",\\\"@type\\\":\\\"Article\\\",\\\"headline\\\":\\\"Components Overview\\\",\\\"image\\\":[\\\"\\\"],\\\"dateModified\\\":null,\\\"author\\\":[]}\"]]},\"headers\":[{\"level\":2,\"title\":\"Components\",\"slug\":\"components\",\"link\":\"#components\",\"children\":[]},{\"level\":2,\"title\":\"Connection pool\",\"slug\":\"connection-pool\",\"link\":\"#connection-pool\",\"children\":[]}],\"filePathRelative\":\"introduction/components_overview.md\",\"autoDesc\":true,\"excerpt\":\"<h2>Components</h2>\\n<ul>\\n<li><code>Connection pool</code>: holds connections in itself and give them when requested.</li>\\n<li><code>Connection</code>: represents single database connection, can be retrieved from <code>Connection pool</code>.</li>\\n<li><code>Transaction</code>: represents database transaction, can be made from <code>Connection</code>.</li>\\n<li><code>Cursor</code>: represents database cursor, can be made from <code>Transaction</code>.</li>\\n<li><code>Results</code>: represents data returned from driver.</li>\\n<li><code>Exceptions</code>: we have some custom exceptions. (Section in development)</li>\\n</ul>\"}")
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
