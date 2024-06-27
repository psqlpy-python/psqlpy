import comp from "/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/usage/types/advanced_type_usage.html.vue"
const data = JSON.parse("{\"path\":\"/usage/types/advanced_type_usage.html\",\"title\":\"Advanced Type Usage\",\"lang\":\"en-US\",\"frontmatter\":{\"title\":\"Advanced Type Usage\",\"description\":\"Due to an unavailability to support all possible types in PostgreSQL, we have a way to encode Python types into PostgreSQL ones and decode wise versa. This section has Advanced ...\",\"head\":[[\"meta\",{\"property\":\"og:url\",\"content\":\"https://qaspen-python.github.io/usage/types/advanced_type_usage.html\"}],[\"meta\",{\"property\":\"og:site_name\",\"content\":\"PSQLPy\"}],[\"meta\",{\"property\":\"og:title\",\"content\":\"Advanced Type Usage\"}],[\"meta\",{\"property\":\"og:description\",\"content\":\"Due to an unavailability to support all possible types in PostgreSQL, we have a way to encode Python types into PostgreSQL ones and decode wise versa. This section has Advanced ...\"}],[\"meta\",{\"property\":\"og:type\",\"content\":\"article\"}],[\"meta\",{\"property\":\"og:locale\",\"content\":\"en-US\"}],[\"script\",{\"type\":\"application/ld+json\"},\"{\\\"@context\\\":\\\"https://schema.org\\\",\\\"@type\\\":\\\"Article\\\",\\\"headline\\\":\\\"Advanced Type Usage\\\",\\\"image\\\":[\\\"\\\"],\\\"dateModified\\\":null,\\\"author\\\":[]}\"]]},\"headers\":[{\"level\":2,\"title\":\"Pass unsupported type into PostgreSQL\",\"slug\":\"pass-unsupported-type-into-postgresql\",\"link\":\"#pass-unsupported-type-into-postgresql\",\"children\":[]},{\"level\":2,\"title\":\"Decode unsupported type from PostgreSQL\",\"slug\":\"decode-unsupported-type-from-postgresql\",\"link\":\"#decode-unsupported-type-from-postgresql\",\"children\":[]}],\"filePathRelative\":\"usage/types/advanced_type_usage.md\",\"autoDesc\":true,\"excerpt\":\"<p>Due to an unavailability to support all possible types in PostgreSQL, we have a way to encode Python types into PostgreSQL ones and decode wise versa.</p>\\n<p>This section has <code>Advanced</code> in the name because you'll need to work with raw bytes which can be difficult for some developers.</p>\"}")
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
