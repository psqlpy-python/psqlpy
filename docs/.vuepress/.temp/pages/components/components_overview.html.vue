<template><div><h2 id="components" tabindex="-1"><a class="header-anchor" href="#components"><span>Components</span></a></h2>
<ul>
<li><code v-pre>ConnectionPool</code>: holds connections in itself and give them when requested.</li>
<li><code v-pre>ConnectionPoolBuilder</code>: Chainable builder for <code v-pre>ConnectionPool</code>, for people who prefer it over big initialization.</li>
<li><code v-pre>Connection</code>: represents single database connection, can be retrieved from <code v-pre>ConnectionPool</code>.</li>
<li><code v-pre>Transaction</code>: represents database transaction, can be made from <code v-pre>Connection</code>.</li>
<li><code v-pre>Cursor</code>: represents database cursor, can be made from <code v-pre>Transaction</code>.</li>
<li><code v-pre>QueryResult</code>: represents list of results from database.</li>
<li><code v-pre>SingleQueryResult</code>: represents single result from the database.</li>
<li><code v-pre>Exceptions</code>: we have some custom exceptions.</li>
</ul>
</div></template>


