import comp from "/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/components/components_overview.html.vue"
const data = JSON.parse("{\"path\":\"/components/components_overview.html\",\"title\":\"Components\",\"lang\":\"en-US\",\"frontmatter\":{\"title\":\"Components\",\"description\":\"Components ConnectionPool: holds connections in itself and give them when requested. ConnectionPoolBuilder: Chainable builder for ConnectionPool, for people who prefer it over b...\",\"head\":[[\"meta\",{\"property\":\"og:url\",\"content\":\"https://psqlpy.github.io/components/components_overview.html\"}],[\"meta\",{\"property\":\"og:site_name\",\"content\":\"PSQLPy\"}],[\"meta\",{\"property\":\"og:title\",\"content\":\"Components\"}],[\"meta\",{\"property\":\"og:description\",\"content\":\"Components ConnectionPool: holds connections in itself and give them when requested. ConnectionPoolBuilder: Chainable builder for ConnectionPool, for people who prefer it over b...\"}],[\"meta\",{\"property\":\"og:type\",\"content\":\"article\"}],[\"meta\",{\"property\":\"og:locale\",\"content\":\"en-US\"}],[\"script\",{\"type\":\"application/ld+json\"},\"{\\\"@context\\\":\\\"https://schema.org\\\",\\\"@type\\\":\\\"Article\\\",\\\"headline\\\":\\\"Components\\\",\\\"image\\\":[\\\"\\\"],\\\"dateModified\\\":null,\\\"author\\\":[]}\"]]},\"headers\":[{\"level\":2,\"title\":\"Components\",\"slug\":\"components\",\"link\":\"#components\",\"children\":[]}],\"filePathRelative\":\"components/components_overview.md\",\"autoDesc\":true,\"excerpt\":\"<h2>Components</h2>\\n<ul>\\n<li><code>ConnectionPool</code>: holds connections in itself and give them when requested.</li>\\n<li><code>ConnectionPoolBuilder</code>: Chainable builder for <code>ConnectionPool</code>, for people who prefer it over big initialization.</li>\\n<li><code>Connection</code>: represents single database connection, can be retrieved from <code>ConnectionPool</code>.</li>\\n<li><code>Transaction</code>: represents database transaction, can be made from <code>Connection</code>.</li>\\n<li><code>Cursor</code>: represents database cursor, can be made from <code>Transaction</code>.</li>\\n<li><code>QueryResult</code>: represents list of results from database.</li>\\n<li><code>SingleQueryResult</code>: represents single result from the database.</li>\\n<li><code>Exceptions</code>: we have some custom exceptions.</li>\\n</ul>\"}")
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
