import comp from "/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/usage/frameworks/fastapi.html.vue"
const data = JSON.parse("{\"path\":\"/usage/frameworks/fastapi.html\",\"title\":\"FastAPI\",\"lang\":\"en-US\",\"frontmatter\":{\"title\":\"FastAPI\",\"description\":\"There is the default example for FastAPI framework. Standard example. This code is perfect for situations when your endpoints don't have complex logic like sending messages over...\",\"head\":[[\"meta\",{\"property\":\"og:url\",\"content\":\"https://qaspen-python.github.io/psqlpy-docs/usage/frameworks/fastapi.html\"}],[\"meta\",{\"property\":\"og:site_name\",\"content\":\"PSQLPy\"}],[\"meta\",{\"property\":\"og:title\",\"content\":\"FastAPI\"}],[\"meta\",{\"property\":\"og:description\",\"content\":\"There is the default example for FastAPI framework. Standard example. This code is perfect for situations when your endpoints don't have complex logic like sending messages over...\"}],[\"meta\",{\"property\":\"og:type\",\"content\":\"article\"}],[\"meta\",{\"property\":\"og:locale\",\"content\":\"en-US\"}],[\"script\",{\"type\":\"application/ld+json\"},\"{\\\"@context\\\":\\\"https://schema.org\\\",\\\"@type\\\":\\\"Article\\\",\\\"headline\\\":\\\"FastAPI\\\",\\\"image\\\":[\\\"\\\"],\\\"dateModified\\\":null,\\\"author\\\":[]}\"]]},\"headers\":[{\"level\":2,\"title\":\"Standard example.\",\"slug\":\"standard-example\",\"link\":\"#standard-example\",\"children\":[]},{\"level\":2,\"title\":\"Advanced example\",\"slug\":\"advanced-example\",\"link\":\"#advanced-example\",\"children\":[]}],\"filePathRelative\":\"usage/frameworks/fastapi.md\",\"autoDesc\":true,\"excerpt\":\"<p>There is the default example for <code>FastAPI</code> framework.</p>\\n<h2>Standard example.</h2>\\n<p>This code is perfect for situations when your endpoints don't have complex logic\\nlike sending messages over network with some queues (<code>RabbitMQ</code>, <code>NATS</code>, <code>Kafka</code> and etc)\\nor making long calculations, so a connection won't idle to much.<br>\\nYou need to take this restrictions into account if you don't have external database connection pool\\nlike <code>PGBouncer</code>.</p>\"}")
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
