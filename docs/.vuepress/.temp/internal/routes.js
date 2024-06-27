export const redirects = JSON.parse("{}")

export const routes = Object.fromEntries([
  ["/", { loader: () => import(/* webpackChunkName: "index.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/index.html.js"), meta: {"t":"PSQLPy documentation","i":"home"} }],
  ["/contribute.html", { loader: () => import(/* webpackChunkName: "contribute.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/contribute.html.js"), meta: {"t":"Contribution guide"} }],
  ["/components/components_overview.html", { loader: () => import(/* webpackChunkName: "components_overview.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/components/components_overview.html.js"), meta: {"t":"Components"} }],
  ["/components/connection.html", { loader: () => import(/* webpackChunkName: "connection.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/components/connection.html.js"), meta: {"t":"Connection"} }],
  ["/components/connection_pool.html", { loader: () => import(/* webpackChunkName: "connection_pool.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/components/connection_pool.html.js"), meta: {"t":"Connection Pool"} }],
  ["/components/connection_pool_builder.html", { loader: () => import(/* webpackChunkName: "connection_pool_builder.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/components/connection_pool_builder.html.js"), meta: {"t":"Connection Pool Builder"} }],
  ["/components/cursor.html", { loader: () => import(/* webpackChunkName: "cursor.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/components/cursor.html.js"), meta: {"t":"Cursor"} }],
  ["/components/exceptions.html", { loader: () => import(/* webpackChunkName: "exceptions.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/components/exceptions.html.js"), meta: {"t":"Exceptions"} }],
  ["/components/results.html", { loader: () => import(/* webpackChunkName: "results.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/components/results.html.js"), meta: {"t":"Results"} }],
  ["/components/transaction.html", { loader: () => import(/* webpackChunkName: "transaction.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/components/transaction.html.js"), meta: {"t":"Transaction"} }],
  ["/introduction/components_overview.html", { loader: () => import(/* webpackChunkName: "components_overview.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/introduction/components_overview.html.js"), meta: {"t":"Components Overview"} }],
  ["/introduction/introduction.html", { loader: () => import(/* webpackChunkName: "introduction.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/introduction/introduction.html.js"), meta: {"t":"What is PSQLPy?"} }],
  ["/introduction/lets_start.html", { loader: () => import(/* webpackChunkName: "lets_start.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/introduction/lets_start.html.js"), meta: {"t":"Let's Start"} }],
  ["/usage/usage.html", { loader: () => import(/* webpackChunkName: "usage.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/usage/usage.html.js"), meta: {"t":"Usage"} }],
  ["/usage/frameworks/aiohttp.html", { loader: () => import(/* webpackChunkName: "aiohttp.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/usage/frameworks/aiohttp.html.js"), meta: {"t":"AioHTTP"} }],
  ["/usage/frameworks/blacksheep.html", { loader: () => import(/* webpackChunkName: "blacksheep.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/usage/frameworks/blacksheep.html.js"), meta: {"t":"Blacksheep"} }],
  ["/usage/frameworks/fastapi.html", { loader: () => import(/* webpackChunkName: "fastapi.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/usage/frameworks/fastapi.html.js"), meta: {"t":"FastAPI"} }],
  ["/usage/frameworks/frameworks.html", { loader: () => import(/* webpackChunkName: "frameworks.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/usage/frameworks/frameworks.html.js"), meta: {"t":"Framework Usage"} }],
  ["/usage/frameworks/litestar.html", { loader: () => import(/* webpackChunkName: "litestar.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/usage/frameworks/litestar.html.js"), meta: {"t":"Litestar"} }],
  ["/usage/frameworks/robyn.html", { loader: () => import(/* webpackChunkName: "robyn.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/usage/frameworks/robyn.html.js"), meta: {"t":"Robyn"} }],
  ["/usage/row_factories/overall_usage.html", { loader: () => import(/* webpackChunkName: "overall_usage.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/usage/row_factories/overall_usage.html.js"), meta: {"t":"Generic usage of row_factory"} }],
  ["/usage/row_factories/predefined_row_factories.html", { loader: () => import(/* webpackChunkName: "predefined_row_factories.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/usage/row_factories/predefined_row_factories.html.js"), meta: {"t":"Predefined row factories"} }],
  ["/usage/row_factories/row_factories.html", { loader: () => import(/* webpackChunkName: "row_factories.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/usage/row_factories/row_factories.html.js"), meta: {"t":"Row Factories Usage"} }],
  ["/usage/types/advanced_type_usage.html", { loader: () => import(/* webpackChunkName: "advanced_type_usage.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/usage/types/advanced_type_usage.html.js"), meta: {"t":"Advanced Type Usage"} }],
  ["/usage/types/extra_types.html", { loader: () => import(/* webpackChunkName: "extra_types.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/usage/types/extra_types.html.js"), meta: {"t":"Extra Types"} }],
  ["/usage/types/supported_types.html", { loader: () => import(/* webpackChunkName: "supported_types.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/usage/types/supported_types.html.js"), meta: {"t":"Supported Types"} }],
  ["/usage/types/types.html", { loader: () => import(/* webpackChunkName: "types.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/usage/types/types.html.js"), meta: {"t":"Types Description"} }],
  ["/404.html", { loader: () => import(/* webpackChunkName: "404.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/404.html.js"), meta: {"t":""} }],
  ["/components/", { loader: () => import(/* webpackChunkName: "index.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/components/index.html.js"), meta: {"t":"Components"} }],
  ["/introduction/", { loader: () => import(/* webpackChunkName: "index.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/introduction/index.html.js"), meta: {"t":"Introduction"} }],
  ["/usage/", { loader: () => import(/* webpackChunkName: "index.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/usage/index.html.js"), meta: {"t":"Usage"} }],
  ["/usage/frameworks/", { loader: () => import(/* webpackChunkName: "index.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/usage/frameworks/index.html.js"), meta: {"t":"Frameworks"} }],
  ["/usage/row_factories/", { loader: () => import(/* webpackChunkName: "index.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/usage/row_factories/index.html.js"), meta: {"t":"Row Factories"} }],
  ["/usage/types/", { loader: () => import(/* webpackChunkName: "index.html" */"/Users/aleksandrkiselev/Projects/qaspen-python/psqlpy/docs/.vuepress/.temp/pages/usage/types/index.html.js"), meta: {"t":"Types"} }],
]);

if (import.meta.webpackHot) {
  import.meta.webpackHot.accept()
  if (__VUE_HMR_RUNTIME__.updateRoutes) {
    __VUE_HMR_RUNTIME__.updateRoutes(routes)
  }
  if (__VUE_HMR_RUNTIME__.updateRedirects) {
    __VUE_HMR_RUNTIME__.updateRedirects(redirects)
  }
}

if (import.meta.hot) {
  import.meta.hot.accept(({ routes, redirects }) => {
    __VUE_HMR_RUNTIME__.updateRoutes(routes)
    __VUE_HMR_RUNTIME__.updateRedirects(redirects)
  })
}
