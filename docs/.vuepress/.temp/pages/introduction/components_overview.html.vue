<template><div><h2 id="components" tabindex="-1"><a class="header-anchor" href="#components"><span>Components</span></a></h2>
<ul>
<li><code v-pre>Connection pool</code>: holds connections in itself and give them when requested.</li>
<li><code v-pre>Connection</code>: represents single database connection, can be retrieved from <code v-pre>Connection pool</code>.</li>
<li><code v-pre>Transaction</code>: represents database transaction, can be made from <code v-pre>Connection</code>.</li>
<li><code v-pre>Cursor</code>: represents database cursor, can be made from <code v-pre>Transaction</code>.</li>
<li><code v-pre>Results</code>: represents data returned from driver.</li>
<li><code v-pre>Exceptions</code>: we have some custom exceptions. (Section in development)</li>
</ul>
<h2 id="connection-pool" tabindex="-1"><a class="header-anchor" href="#connection-pool"><span>Connection pool</span></a></h2>
<p>Connection pool is the main object in the library. It initializes, creates, holds and gives connection to the user side.<br>
Connection pool must be started up before any other operations.</p>
</div></template>


