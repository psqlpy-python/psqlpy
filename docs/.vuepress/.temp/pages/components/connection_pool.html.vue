<template><div><p>Connection pool is the main object in the library. It initializes, creates, holds and gives connection to the user side.<br>
Connection pool must be started up before any other operations.</p>
<div class="hint-container important">
<p class="hint-container-title">Important</p>
<p>You cannot set the minimum size for the connection pool, by it is 0.</p>
<p><code v-pre>ConnectionPool</code> doesn't create connection on startup. It makes new connection on demand.</p>
<p>So, if you set <code v-pre>max_db_pool_size</code> to 100, pool will create new connection every time there aren't enough connections to handle the load.</p>
</div>
<h2 id="connection-pool-methods" tabindex="-1"><a class="header-anchor" href="#connection-pool-methods"><span>Connection pool methods</span></a></h2>
<h3 id="all-available-connectionpool-parameters" tabindex="-1"><a class="header-anchor" href="#all-available-connectionpool-parameters"><span>All available ConnectionPool parameters</span></a></h3>
<ul>
<li><code v-pre>dsn</code>: Full dsn connection string.
<code v-pre>postgres://postgres:postgres@localhost:5432/postgres?target_session_attrs=read-write</code></li>
<li><code v-pre>username</code>: Username of the user in the <code v-pre>PostgreSQL</code></li>
<li><code v-pre>password</code>: Password of the user in the <code v-pre>PostgreSQL</code></li>
<li><code v-pre>host</code>: Host of the <code v-pre>PostgreSQL</code></li>
<li><code v-pre>hosts</code>: Hosts of the <code v-pre>PostgreSQL</code></li>
<li><code v-pre>port</code>: Port of the <code v-pre>PostgreSQL</code></li>
<li><code v-pre>ports</code>: Ports of the <code v-pre>PostgreSQL</code></li>
<li><code v-pre>db_name</code>: Name of the database in <code v-pre>PostgreSQL</code></li>
<li><code v-pre>target_session_attrs</code>: Specifies requirements of the session.</li>
<li><code v-pre>options</code>: Command line options used to configure the server</li>
<li><code v-pre>application_name</code>: Sets the application_name parameter on the server.</li>
<li><code v-pre>connect_timeout_sec</code>: The time limit in seconds applied to each socket-level
connection attempt.
Note that hostnames can resolve to multiple IP addresses,
and this limit is applied to each address. Defaults to no timeout.</li>
<li><code v-pre>connect_timeout_nanosec</code>: nanosec for connection timeout,
can be used only with connect_timeout_sec.</li>
<li><code v-pre>tcp_user_timeout_sec</code>: The time limit that
transmitted data may remain unacknowledged
before a connection is forcibly closed.
This is ignored for Unix domain socket connections.
It is only supported on systems where TCP_USER_TIMEOUT
is available and will default to the system default if omitted
or set to 0; on other systems, it has no effect.</li>
<li><code v-pre>tcp_user_timeout_nanosec</code>: nanosec for cp_user_timeout,
can be used only with tcp_user_timeout_sec.</li>
<li><code v-pre>keepalives</code>: Controls the use of TCP keepalive.
This option is ignored when connecting with Unix sockets.
Defaults to on.</li>
<li><code v-pre>keepalives_idle_sec</code>: The number of seconds of inactivity after
which a keepalive message is sent to the server.
This option is ignored when connecting with Unix sockets.
Defaults to 2 hours.</li>
<li><code v-pre>keepalives_idle_nanosec</code>: Nanosec for keepalives_idle_sec.</li>
<li><code v-pre>keepalives_interval_sec</code>: The time interval between TCP keepalive probes.
This option is ignored when connecting with Unix sockets.</li>
<li><code v-pre>keepalives_interval_nanosec</code>: Nanosec for keepalives_interval_sec.</li>
<li><code v-pre>keepalives_retries</code>: The maximum number of TCP keepalive probes
that will be sent before dropping a connection.
This option is ignored when connecting with Unix sockets.</li>
<li><code v-pre>load_balance_hosts</code>: Controls the order in which the client tries to connect
to the available hosts and addresses.
Once a connection attempt is successful no other
hosts and addresses will be tried.
This parameter is typically used in combination with multiple host names
or a DNS record that returns multiple IPs.
If set to disable, hosts and addresses will be tried in the order provided.
If set to random, hosts will be tried in a random order, and the IP addresses
resolved from a hostname will also be tried in a random order.
Defaults to disable.</li>
<li><code v-pre>max_db_pool_size</code>: maximum size of the connection pool.</li>
<li><code v-pre>conn_recycling_method</code>: how a connection is recycled.</li>
</ul>
<p>Example of possible <code v-pre>dsn</code>s:</p>
<div class="language-text line-numbers-mode" data-ext="text" data-title="text"><pre v-pre class="language-text"><code>postgresql://user@localhost
postgresql://user:password@%2Fvar%2Flib%2Fpostgresql/mydb?connect_timeout=10
postgresql://user@host1:1234,host2,host3:5678?target_session_attrs=read-write
postgresql:///mydb?user=user&amp;host=/var/lib/postgresql
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><div class="hint-container important">
<p class="hint-container-title">Important</p>
<p>If <code v-pre>dsn</code> is specified then <code v-pre>username</code>, <code v-pre>password</code>, <code v-pre>host</code>, <code v-pre>hosts</code>, <code v-pre>port</code>, <code v-pre>ports</code>, <code v-pre>db_name</code> and <code v-pre>target_session_attrs</code>
parameters will be ignored.</p>
</div>
<h3 id="initialize-connection-pool-with-separate-parameters" tabindex="-1"><a class="header-anchor" href="#initialize-connection-pool-with-separate-parameters"><span>Initialize Connection Pool with separate parameters</span></a></h3>
<p>There are two ways of how to connect to the database. First one is use connection parameters separately:</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">import</span> asyncio
<span class="token keyword">from</span> typing <span class="token keyword">import</span> Final

<span class="token keyword">from</span> psqlpy <span class="token keyword">import</span> ConnectionPool

db_pool<span class="token punctuation">:</span> Final <span class="token operator">=</span> ConnectionPool<span class="token punctuation">(</span>
    username<span class="token operator">=</span><span class="token string">"postgres"</span><span class="token punctuation">,</span>
    password<span class="token operator">=</span><span class="token string">"postgres"</span><span class="token punctuation">,</span>
    host<span class="token operator">=</span><span class="token string">"localhost"</span><span class="token punctuation">,</span>
    port<span class="token operator">=</span><span class="token number">5432</span><span class="token punctuation">,</span>
    db_name<span class="token operator">=</span><span class="token string">"postgres"</span><span class="token punctuation">,</span>
    max_db_pool_size<span class="token operator">=</span><span class="token number">10</span><span class="token punctuation">,</span>
<span class="token punctuation">)</span>

<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>

</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="initialize-connection-pool-with-dsn" tabindex="-1"><a class="header-anchor" href="#initialize-connection-pool-with-dsn"><span>Initialize Connection Pool with DSN</span></a></h3>
<p>Other way is use DSN:</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">import</span> asyncio
<span class="token keyword">from</span> typing <span class="token keyword">import</span> Final

<span class="token keyword">from</span> psqlpy <span class="token keyword">import</span> ConnectionPool

db_pool<span class="token punctuation">:</span> Final <span class="token operator">=</span> ConnectionPool<span class="token punctuation">(</span>
    dsn<span class="token operator">=</span><span class="token string">"postgres://postgres:postgres@localhost:5432/postgres"</span><span class="token punctuation">,</span>
    max_db_pool_size<span class="token operator">=</span><span class="token number">10</span><span class="token punctuation">,</span>
<span class="token punctuation">)</span>

<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>

</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="create-connection-pool-with-one-function" tabindex="-1"><a class="header-anchor" href="#create-connection-pool-with-one-function"><span>Create Connection Pool with one function</span></a></h3>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">from</span> typing <span class="token keyword">import</span> Final

<span class="token keyword">from</span> psqlpy <span class="token keyword">import</span> connect


db_pool<span class="token punctuation">:</span> Final <span class="token operator">=</span> connect<span class="token punctuation">(</span>
    dsn<span class="token operator">=</span><span class="token string">"postgres://postgres:postgres@localhost:5432/postgres"</span><span class="token punctuation">,</span>
    max_db_pool_size<span class="token operator">=</span><span class="token number">10</span><span class="token punctuation">,</span>
<span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><p><code v-pre>connect</code> function has the same parameters as <code v-pre>ConnectionPool</code>.</p>
<h3 id="execute" tabindex="-1"><a class="header-anchor" href="#execute"><span>Execute</span></a></h3>
<h4 id="parameters" tabindex="-1"><a class="header-anchor" href="#parameters"><span>Parameters:</span></a></h4>
<ul>
<li><code v-pre>querystring</code>: Statement string.</li>
<li><code v-pre>parameters</code>: List of parameters for the statement string.</li>
<li><code v-pre>prepared</code>: Prepare statement before execution or not.</li>
</ul>
<p>You can execute any query directly from Connection Pool.<br>
This method supports parameters, each parameter must be marked as <code v-pre>$&lt;number&gt;</code> (number starts with 1).<br>
Parameters must be passed as list after querystring.</p>
<div class="hint-container caution">
<p class="hint-container-title">Caution</p>
<p>You must use <code v-pre>ConnectionPool.execute</code> method in high-load production code wisely!<br>
It pulls connection from the pool each time you execute query.<br>
Preferable way to execute statements with <RouteLink to="/introduction/components/connection.html">Connection</RouteLink> or <RouteLink to="/introduction/components/transaction.html">Transaction</RouteLink></p>
</div>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span>
    results<span class="token punctuation">:</span> QueryResult <span class="token operator">=</span> <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>execute<span class="token punctuation">(</span>
        <span class="token string">"SELECT * FROM users WHERE id = $1 and username = $2"</span><span class="token punctuation">,</span>
        <span class="token punctuation">[</span><span class="token number">100</span><span class="token punctuation">,</span> <span class="token string">"Alex"</span><span class="token punctuation">]</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>

    dict_results<span class="token punctuation">:</span> <span class="token builtin">list</span><span class="token punctuation">[</span><span class="token builtin">dict</span><span class="token punctuation">[</span><span class="token builtin">str</span><span class="token punctuation">,</span> Any<span class="token punctuation">]</span><span class="token punctuation">]</span> <span class="token operator">=</span> results<span class="token punctuation">.</span>result<span class="token punctuation">(</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="fetch" tabindex="-1"><a class="header-anchor" href="#fetch"><span>Fetch</span></a></h3>
<h4 id="parameters-1" tabindex="-1"><a class="header-anchor" href="#parameters-1"><span>Parameters:</span></a></h4>
<ul>
<li><code v-pre>querystring</code>: Statement string.</li>
<li><code v-pre>parameters</code>: List of parameters for the statement string.</li>
<li><code v-pre>prepared</code>: Prepare statement before execution or not.</li>
</ul>
<p>The same as the <code v-pre>execute</code> method, for some people this naming is preferable.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span>
    results<span class="token punctuation">:</span> QueryResult <span class="token operator">=</span> <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>fetch<span class="token punctuation">(</span>
        <span class="token string">"SELECT * FROM users WHERE id = $1 and username = $2"</span><span class="token punctuation">,</span>
        <span class="token punctuation">[</span><span class="token number">100</span><span class="token punctuation">,</span> <span class="token string">"Alex"</span><span class="token punctuation">]</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>

    dict_results<span class="token punctuation">:</span> <span class="token builtin">list</span><span class="token punctuation">[</span><span class="token builtin">dict</span><span class="token punctuation">[</span><span class="token builtin">str</span><span class="token punctuation">,</span> Any<span class="token punctuation">]</span><span class="token punctuation">]</span> <span class="token operator">=</span> results<span class="token punctuation">.</span>result<span class="token punctuation">(</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="connection" tabindex="-1"><a class="header-anchor" href="#connection"><span>Connection</span></a></h3>
<p>To get single connection from the <code v-pre>ConnectionPool</code> there is method named <code v-pre>connection()</code>.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span>
    connection <span class="token operator">=</span> <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>connection<span class="token punctuation">(</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><div class="hint-container tip">
<p class="hint-container-title">Cool tip</p>
<p>This is the preferable way to work with the PostgreSQL.</p>
</div>
<h3 id="close" tabindex="-1"><a class="header-anchor" href="#close"><span>Close</span></a></h3>
<p>To close the connection pool at the stop of your application.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span>
    db_pool<span class="token punctuation">.</span>close<span class="token punctuation">(</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div></div></template>


