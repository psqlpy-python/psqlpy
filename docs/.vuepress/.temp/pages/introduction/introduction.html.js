import comp from "/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/introduction/introduction.html.vue"
const data = JSON.parse("{\"path\":\"/introduction/introduction.html\",\"title\":\"What is PSQLPy?\",\"lang\":\"en-US\",\"frontmatter\":{\"title\":\"What is PSQLPy?\",\"description\":\"PSQLPy is a new Python driver for PostgreSQL fully written in Rust. It was inspired by Psycopg3 and AsyncPG. With PSQLPy you can: Make an interaction with the PostgeSQL in your ...\",\"head\":[[\"meta\",{\"property\":\"og:url\",\"content\":\"https://qaspen-python.github.io/introduction/introduction.html\"}],[\"meta\",{\"property\":\"og:site_name\",\"content\":\"PSQLPy\"}],[\"meta\",{\"property\":\"og:title\",\"content\":\"What is PSQLPy?\"}],[\"meta\",{\"property\":\"og:description\",\"content\":\"PSQLPy is a new Python driver for PostgreSQL fully written in Rust. It was inspired by Psycopg3 and AsyncPG. With PSQLPy you can: Make an interaction with the PostgeSQL in your ...\"}],[\"meta\",{\"property\":\"og:type\",\"content\":\"article\"}],[\"meta\",{\"property\":\"og:locale\",\"content\":\"en-US\"}],[\"script\",{\"type\":\"application/ld+json\"},\"{\\\"@context\\\":\\\"https://schema.org\\\",\\\"@type\\\":\\\"Article\\\",\\\"headline\\\":\\\"What is PSQLPy?\\\",\\\"image\\\":[\\\"\\\"],\\\"dateModified\\\":null,\\\"author\\\":[]}\"]]},\"headers\":[{\"level\":2,\"title\":\"Important notes\",\"slug\":\"important-notes\",\"link\":\"#important-notes\",\"children\":[]},{\"level\":2,\"title\":\"Join community!\",\"slug\":\"join-community\",\"link\":\"#join-community\",\"children\":[]}],\"filePathRelative\":\"introduction/introduction.md\",\"autoDesc\":true,\"excerpt\":\"<p><code>PSQLPy</code> is a new Python driver for PostgreSQL fully written in Rust. It was inspired by <code>Psycopg3</code> and <code>AsyncPG</code>.</p>\\n<p>With <code>PSQLPy</code> you can:</p>\\n<ul>\\n<li>Make an interaction with the PostgeSQL in your application much faster (2-3 times).</li>\\n<li>Be sure that there won't be any unexpected errors.</li>\\n<li>Don't usually go to the documentation to search every question - we have awesome docstrings for every component.</li>\\n<li>Use <code>MyPy</code> (or any other Python type checker) with confidence that exactly the types specified in the typing will be returned.</li>\\n<li>Concentrate on writing your code, not understanding new abstractions in this library, we only have classes which represents PostgreSQL object (transaction, cursor, etc).</li>\\n</ul>\"}")
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
