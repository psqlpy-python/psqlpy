<template><div><p><code v-pre>Connection</code> object represents single connection to the <code v-pre>PostgreSQL</code>. You must work with database within it.<br>
<code v-pre>Connection</code> get be made with <code v-pre>ConnectionPool().connection()</code> method.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">from</span> psqlpy <span class="token keyword">import</span> ConnectionPool


db_pool<span class="token punctuation">:</span> Final <span class="token operator">=</span> ConnectionPool<span class="token punctuation">(</span>
    dsn<span class="token operator">=</span><span class="token string">"postgres://postgres:postgres@localhost:5432/postgres"</span><span class="token punctuation">,</span>
<span class="token punctuation">)</span>


<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span>
    connection <span class="token operator">=</span> <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>connection<span class="token punctuation">(</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h2 id="connection-methods" tabindex="-1"><a class="header-anchor" href="#connection-methods"><span>Connection methods</span></a></h2>
<h3 id="execute" tabindex="-1"><a class="header-anchor" href="#execute"><span>Execute</span></a></h3>
<h4 id="parameters" tabindex="-1"><a class="header-anchor" href="#parameters"><span>Parameters:</span></a></h4>
<ul>
<li><code v-pre>querystring</code>: Statement string.</li>
<li><code v-pre>parameters</code>: List of parameters for the statement string.</li>
<li><code v-pre>prepared</code>: Prepare statement before execution or not.</li>
</ul>
<p>You can execute any query directly from <code v-pre>Connection</code> object.<br>
This method supports parameters, each parameter must be marked as <code v-pre>$&lt;number&gt;</code> in querystring (number starts with 1).</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span>
    connection <span class="token operator">=</span> <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>connection<span class="token punctuation">(</span><span class="token punctuation">)</span>
    results<span class="token punctuation">:</span> QueryResult <span class="token operator">=</span> <span class="token keyword">await</span> connection<span class="token punctuation">.</span>execute<span class="token punctuation">(</span>
        <span class="token string">"SELECT * FROM users WHERE id = $1 and username = $2"</span><span class="token punctuation">,</span>
        <span class="token punctuation">[</span><span class="token number">100</span><span class="token punctuation">,</span> <span class="token string">"Alex"</span><span class="token punctuation">]</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>

    dict_results<span class="token punctuation">:</span> <span class="token builtin">list</span><span class="token punctuation">[</span><span class="token builtin">dict</span><span class="token punctuation">[</span><span class="token builtin">str</span><span class="token punctuation">,</span> Any<span class="token punctuation">]</span><span class="token punctuation">]</span> <span class="token operator">=</span> results<span class="token punctuation">.</span>result<span class="token punctuation">(</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="fetch" tabindex="-1"><a class="header-anchor" href="#fetch"><span>Fetch</span></a></h3>
<h4 id="parameters-1" tabindex="-1"><a class="header-anchor" href="#parameters-1"><span>Parameters:</span></a></h4>
<ul>
<li><code v-pre>querystring</code>: Statement string.</li>
<li><code v-pre>parameters</code>: List of parameters for the statement string.</li>
<li><code v-pre>prepared</code>: Prepare statement before execution or not.</li>
</ul>
<p>The same as the <code v-pre>execute</code> method, for some people this naming is preferable.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span>
    connection <span class="token operator">=</span> <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>connection<span class="token punctuation">(</span><span class="token punctuation">)</span>
    results<span class="token punctuation">:</span> QueryResult <span class="token operator">=</span> <span class="token keyword">await</span> connection<span class="token punctuation">.</span>fetch<span class="token punctuation">(</span>
        <span class="token string">"SELECT * FROM users WHERE id = $1 and username = $2"</span><span class="token punctuation">,</span>
        <span class="token punctuation">[</span><span class="token number">100</span><span class="token punctuation">,</span> <span class="token string">"Alex"</span><span class="token punctuation">]</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>

    dict_results<span class="token punctuation">:</span> <span class="token builtin">list</span><span class="token punctuation">[</span><span class="token builtin">dict</span><span class="token punctuation">[</span><span class="token builtin">str</span><span class="token punctuation">,</span> Any<span class="token punctuation">]</span><span class="token punctuation">]</span> <span class="token operator">=</span> results<span class="token punctuation">.</span>result<span class="token punctuation">(</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="execute-many" tabindex="-1"><a class="header-anchor" href="#execute-many"><span>Execute Many</span></a></h3>
<h4 id="parameters-2" tabindex="-1"><a class="header-anchor" href="#parameters-2"><span>Parameters:</span></a></h4>
<ul>
<li><code v-pre>querystring</code>: Statement string.</li>
<li><code v-pre>parameters</code>: List of list of parameters for the statement string.</li>
<li><code v-pre>prepared</code>: Prepare statement before execution or not.</li>
</ul>
<p>This method supports parameters, each parameter must be marked as <code v-pre>$&lt;number&gt;</code> in querystring (number starts with 1).
Atomicity is provided, so you don't need to worry about unsuccessful result, because there is a transaction used internally.
This method returns nothing.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span>
    connection <span class="token operator">=</span> <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>connection<span class="token punctuation">(</span><span class="token punctuation">)</span>
    <span class="token keyword">await</span> connection<span class="token punctuation">.</span>execute_many<span class="token punctuation">(</span>
        <span class="token string">"INSERT INTO users (name, age) VALUES ($1, $2)"</span><span class="token punctuation">,</span>
        <span class="token punctuation">[</span><span class="token punctuation">[</span><span class="token string">"boba"</span><span class="token punctuation">,</span> <span class="token number">10</span><span class="token punctuation">]</span><span class="token punctuation">,</span> <span class="token punctuation">[</span><span class="token string">"boba"</span><span class="token punctuation">,</span> <span class="token number">20</span><span class="token punctuation">]</span><span class="token punctuation">]</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="fetch-row" tabindex="-1"><a class="header-anchor" href="#fetch-row"><span>Fetch Row</span></a></h3>
<h4 id="parameters-3" tabindex="-1"><a class="header-anchor" href="#parameters-3"><span>Parameters:</span></a></h4>
<ul>
<li><code v-pre>querystring</code>: Statement string.</li>
<li><code v-pre>parameters</code>: List of list of parameters for the statement string.</li>
<li><code v-pre>prepared</code>: Prepare statements before execution or not.</li>
</ul>
<p>Sometimes you need to fetch only first row from the result.</p>
<div class="hint-container warning">
<p class="hint-container-title">Warning</p>
<p>Querystring must return exactly one result or an exception will be raised.</p>
</div>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span>
    connection <span class="token operator">=</span> <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>connection<span class="token punctuation">(</span><span class="token punctuation">)</span>
    query_result<span class="token punctuation">:</span> SingleQueryResult <span class="token operator">=</span> <span class="token keyword">await</span> transaction<span class="token punctuation">.</span>fetch_row<span class="token punctuation">(</span>
        <span class="token string">"SELECT username FROM users WHERE id = $1"</span><span class="token punctuation">,</span>
        <span class="token punctuation">[</span><span class="token number">100</span><span class="token punctuation">]</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>
    dict_result<span class="token punctuation">:</span> Dict<span class="token punctuation">[</span>Any<span class="token punctuation">,</span> Any<span class="token punctuation">]</span> <span class="token operator">=</span> query_result<span class="token punctuation">.</span>result<span class="token punctuation">(</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="fetch-val" tabindex="-1"><a class="header-anchor" href="#fetch-val"><span>Fetch Val</span></a></h3>
<h4 id="parameters-4" tabindex="-1"><a class="header-anchor" href="#parameters-4"><span>Parameters</span></a></h4>
<ul>
<li><code v-pre>querystring</code>: Statement string.</li>
<li><code v-pre>parameters</code>: List of list of parameters for the statement string.</li>
<li><code v-pre>prepared</code>: Prepare statements before execution or not.</li>
</ul>
<p>If you need to retrieve some value not <code v-pre>QueryResult</code>.</p>
<div class="hint-container warning">
<p class="hint-container-title">Warning</p>
<p>Querystring must return exactly one result or an exception will be raised.</p>
</div>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span>
    connection <span class="token operator">=</span> <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>connection<span class="token punctuation">(</span><span class="token punctuation">)</span>
    <span class="token comment"># this will be an int value</span>
    query_result_value <span class="token operator">=</span> <span class="token keyword">await</span> connection<span class="token punctuation">.</span>fetch_row<span class="token punctuation">(</span>
        <span class="token string">"SELECT COUNT(*) FROM users WHERE id > $1"</span><span class="token punctuation">,</span>
        <span class="token punctuation">[</span><span class="token number">100</span><span class="token punctuation">]</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="transaction" tabindex="-1"><a class="header-anchor" href="#transaction"><span>Transaction</span></a></h3>
<p><code v-pre>Connection</code> is the only object that can be used to build <code v-pre>Transaction</code> object.</p>
<h4 id="parameters-5" tabindex="-1"><a class="header-anchor" href="#parameters-5"><span>Parameters:</span></a></h4>
<ul>
<li><code v-pre>isolation_level</code>: level of isolation. Default how it is in PostgreSQL.</li>
<li><code v-pre>read_variant</code>: configure read variant of the transaction. Default how it is in PostgreSQL.</li>
<li><code v-pre>deferrable</code>: configure deferrable of the transaction. Default how it is in PostgreSQL.</li>
</ul>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">from</span> psqlpy <span class="token keyword">import</span> IsolationLevel<span class="token punctuation">,</span> ReadVariant

<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span>
    connection <span class="token operator">=</span> <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>connection<span class="token punctuation">(</span><span class="token punctuation">)</span>
    transaction <span class="token operator">=</span> connection<span class="token punctuation">.</span>transaction<span class="token punctuation">(</span>
        isolation_level<span class="token operator">=</span>IsolationLevel<span class="token punctuation">.</span>Serializable<span class="token punctuation">,</span>
        read_variant<span class="token operator">=</span>ReadVariant<span class="token punctuation">.</span>ReadWrite<span class="token punctuation">,</span>
        deferrable<span class="token operator">=</span><span class="token boolean">True</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div></div></template>


