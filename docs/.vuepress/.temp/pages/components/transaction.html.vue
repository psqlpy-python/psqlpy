<template><div><p><code v-pre>Transaction</code> object represents <code v-pre>PostgreSQL</code> transaction.<br>
There are two ways of how we can work with transactions on <code v-pre>PSQLPy</code> side.</p>
<h3 id="transaction-parameters" tabindex="-1"><a class="header-anchor" href="#transaction-parameters"><span>Transaction parameters</span></a></h3>
<ul>
<li><code v-pre>isolation_level</code>: level of isolation. Default how it is in PostgreSQL.</li>
<li><code v-pre>read_variant</code>: configure read variant of the transaction. Default how it is in PostgreSQL.</li>
<li><code v-pre>deferrable</code>: configure deferrable of the transaction. Default how it is in PostgreSQL.</li>
</ul>
<h3 id="control-transaction-fully-on-your-own" tabindex="-1"><a class="header-anchor" href="#control-transaction-fully-on-your-own"><span>Control transaction fully on your own.</span></a></h3>
<p>First of all, you can get transaction object only from connection object.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">from</span> psqlpy <span class="token keyword">import</span> ConnectionPool


db_pool<span class="token punctuation">:</span> Final <span class="token operator">=</span> ConnectionPool<span class="token punctuation">(</span>
    dsn<span class="token operator">=</span><span class="token string">"postgres://postgres:postgres@localhost:5432/postgres"</span><span class="token punctuation">,</span>
<span class="token punctuation">)</span>


<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    connection <span class="token operator">=</span> <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>connection<span class="token punctuation">(</span><span class="token punctuation">)</span>
    transaction <span class="token operator">=</span> connection<span class="token punctuation">.</span>transaction<span class="token punctuation">(</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><p>After this you need to start you transaction or in <code v-pre>PostgreSQL</code> terms you need to BEGIN it.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span>
    connection <span class="token operator">=</span> <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>connection<span class="token punctuation">(</span><span class="token punctuation">)</span>
    transaction <span class="token operator">=</span> connection<span class="token punctuation">.</span>transaction<span class="token punctuation">(</span><span class="token punctuation">)</span>
    <span class="token keyword">await</span> transaction<span class="token punctuation">.</span>begin<span class="token punctuation">(</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><p>So, after these manipulations you are ready to make you first query with the transaction.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span>
    <span class="token keyword">await</span> transaction<span class="token punctuation">.</span>execute<span class="token punctuation">(</span>
        <span class="token string">"INSERT INTO users (id, username) VALUES ($1, $2)"</span><span class="token punctuation">,</span>
        <span class="token punctuation">[</span><span class="token string">"100"</span><span class="token punctuation">,</span> <span class="token string">"Alex"</span><span class="token punctuation">]</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><p>Good! We've inserted our first row, but if we won't commit the transaction all changes will discard.</p>
<div class="hint-container warning">
<p class="hint-container-title">Warning</p>
<p>We need to commit changes.</p>
</div>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span>
    <span class="token keyword">await</span> transaction<span class="token punctuation">.</span>commit<span class="token punctuation">(</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><p>So, now everything is fine, changes are committed. But you can say that it's too complicated and you are right!<br>
We have an alternative way to handle <code v-pre>begin()</code> and <code v-pre>commit()</code> automatically.</p>
<h3 id="control-transaction-with-async-context-manager" tabindex="-1"><a class="header-anchor" href="#control-transaction-with-async-context-manager"><span>Control transaction with async context manager.</span></a></h3>
<p>There is the previous example but it is rewritten with use of async context manager.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">from</span> psqlpy <span class="token keyword">import</span> ConnectionPool


db_pool<span class="token punctuation">:</span> Final <span class="token operator">=</span> ConnectionPool<span class="token punctuation">(</span>
    dsn<span class="token operator">=</span><span class="token string">"postgres://postgres:postgres@localhost:5432/postgres"</span><span class="token punctuation">,</span>
<span class="token punctuation">)</span>


<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>startup<span class="token punctuation">(</span><span class="token punctuation">)</span>
    connection <span class="token operator">=</span> <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>connection<span class="token punctuation">(</span><span class="token punctuation">)</span>
    <span class="token keyword">async</span> <span class="token keyword">with</span> connection<span class="token punctuation">.</span>transaction<span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token keyword">as</span> transaction<span class="token punctuation">:</span>
        <span class="token comment"># begin() calls automatically</span>
        <span class="token keyword">await</span> transaction<span class="token punctuation">.</span>execute<span class="token punctuation">(</span>
            <span class="token string">"INSERT INTO users (id, username) VALUES ($1, $2)"</span><span class="token punctuation">,</span>
            <span class="token punctuation">[</span><span class="token string">"100"</span><span class="token punctuation">,</span> <span class="token string">"Alex"</span><span class="token punctuation">]</span><span class="token punctuation">,</span>
        <span class="token punctuation">)</span>
        <span class="token comment"># commit() calls automatically.</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><div class="hint-container tip">
<p class="hint-container-title">Cool tip</p>
<p>If a query raises an error in our async context manager, <code v-pre>ROLLBACK</code> is executed automatically.</p>
</div>
<div class="hint-container important">
<p class="hint-container-title">Important</p>
<p>Transaction can be began only once, so if you have already called <code v-pre>begin()</code> manually then async context manager initialize will fail, you need to choose what to use.</p>
</div>
<h2 id="transaction-methods" tabindex="-1"><a class="header-anchor" href="#transaction-methods"><span>Transaction methods</span></a></h2>
<h3 id="begin" tabindex="-1"><a class="header-anchor" href="#begin"><span>Begin</span></a></h3>
<p>You can start a transaction manually.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span>
    <span class="token keyword">await</span> transaction<span class="token punctuation">.</span>begin<span class="token punctuation">(</span><span class="token punctuation">)</span>
    <span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="commit" tabindex="-1"><a class="header-anchor" href="#commit"><span>Commit</span></a></h3>
<p>You can commit a transaction manually.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span>
    <span class="token keyword">await</span> transaction<span class="token punctuation">.</span>commit<span class="token punctuation">(</span><span class="token punctuation">)</span>
    <span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="execute" tabindex="-1"><a class="header-anchor" href="#execute"><span>Execute</span></a></h3>
<h4 id="parameters" tabindex="-1"><a class="header-anchor" href="#parameters"><span>Parameters:</span></a></h4>
<ul>
<li><code v-pre>querystring</code>: Statement string.</li>
<li><code v-pre>parameters</code>: List of parameters for the statement string.</li>
<li><code v-pre>prepared</code>: Prepare statement before execution or not.</li>
</ul>
<p>You can execute any query directly from <code v-pre>Transaction</code> object.<br>
This method supports parameters, each parameter must be marked as <code v-pre>$&lt;number&gt;</code> (number starts with 1).</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span>
    connection <span class="token operator">=</span> <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>connection<span class="token punctuation">(</span><span class="token punctuation">)</span>
    <span class="token keyword">async</span> <span class="token keyword">with</span> connection<span class="token punctuation">.</span>transaction<span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token keyword">as</span> transaction<span class="token punctuation">:</span>
        results<span class="token punctuation">:</span> QueryResult <span class="token operator">=</span> <span class="token keyword">await</span> transaction<span class="token punctuation">.</span>execute<span class="token punctuation">(</span>
            querystring<span class="token operator">=</span><span class="token string">"SELECT * FROM users WHERE id = $1 and username = $2"</span><span class="token punctuation">,</span>
            parameters<span class="token operator">=</span><span class="token punctuation">[</span><span class="token number">100</span><span class="token punctuation">,</span> <span class="token string">"Alex"</span><span class="token punctuation">]</span><span class="token punctuation">,</span>
        <span class="token punctuation">)</span>

    dict_results<span class="token punctuation">:</span> <span class="token builtin">list</span><span class="token punctuation">[</span><span class="token builtin">dict</span><span class="token punctuation">[</span><span class="token builtin">str</span><span class="token punctuation">,</span> Any<span class="token punctuation">]</span><span class="token punctuation">]</span> <span class="token operator">=</span> results<span class="token punctuation">.</span>result<span class="token punctuation">(</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="fetch" tabindex="-1"><a class="header-anchor" href="#fetch"><span>Fetch</span></a></h3>
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
    <span class="token keyword">async</span> <span class="token keyword">with</span> connection<span class="token punctuation">.</span>transaction<span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token keyword">as</span> transaction<span class="token punctuation">:</span>
        results<span class="token punctuation">:</span> QueryResult <span class="token operator">=</span> <span class="token keyword">await</span> transaction<span class="token punctuation">.</span>fetch<span class="token punctuation">(</span>
            querystring<span class="token operator">=</span><span class="token string">"SELECT * FROM users WHERE id = $1 and username = $2"</span><span class="token punctuation">,</span>
            parameters<span class="token operator">=</span><span class="token punctuation">[</span><span class="token number">100</span><span class="token punctuation">,</span> <span class="token string">"Alex"</span><span class="token punctuation">]</span><span class="token punctuation">,</span>
        <span class="token punctuation">)</span>

    dict_results<span class="token punctuation">:</span> <span class="token builtin">list</span><span class="token punctuation">[</span><span class="token builtin">dict</span><span class="token punctuation">[</span><span class="token builtin">str</span><span class="token punctuation">,</span> Any<span class="token punctuation">]</span><span class="token punctuation">]</span> <span class="token operator">=</span> results<span class="token punctuation">.</span>result<span class="token punctuation">(</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="execute-many" tabindex="-1"><a class="header-anchor" href="#execute-many"><span>Execute Many</span></a></h3>
<h4 id="parameters-2" tabindex="-1"><a class="header-anchor" href="#parameters-2"><span>Parameters:</span></a></h4>
<ul>
<li><code v-pre>querystring</code>: Statement string.</li>
<li><code v-pre>parameters</code>: List of list of parameters for the statement string.</li>
<li><code v-pre>prepared</code>: Prepare statements before execution or not.</li>
</ul>
<p>If you want to execute the same querystring, but with different parameters, <code v-pre>execute_many</code> is for you!</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span>
    connection <span class="token operator">=</span> <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>connection<span class="token punctuation">(</span><span class="token punctuation">)</span>
    <span class="token keyword">async</span> <span class="token keyword">with</span> connection<span class="token punctuation">.</span>transaction<span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token keyword">as</span> transaction<span class="token punctuation">:</span>
        <span class="token keyword">await</span> transaction<span class="token punctuation">.</span>execute_many<span class="token punctuation">(</span>
            <span class="token string">"INSERT INTO users (name, age) VALUES ($1, $2)"</span><span class="token punctuation">,</span>
            <span class="token punctuation">[</span><span class="token punctuation">[</span><span class="token string">"boba"</span><span class="token punctuation">,</span> <span class="token number">10</span><span class="token punctuation">]</span><span class="token punctuation">,</span> <span class="token punctuation">[</span><span class="token string">"biba"</span><span class="token punctuation">,</span> <span class="token number">20</span><span class="token punctuation">]</span><span class="token punctuation">]</span><span class="token punctuation">,</span>
        <span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="fetch-row" tabindex="-1"><a class="header-anchor" href="#fetch-row"><span>Fetch Row</span></a></h3>
<h4 id="parameters-3" tabindex="-1"><a class="header-anchor" href="#parameters-3"><span>Parameters</span></a></h4>
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
    <span class="token keyword">async</span> <span class="token keyword">with</span> connection<span class="token punctuation">.</span>transaction<span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token keyword">as</span> transaction<span class="token punctuation">:</span>
        query_result<span class="token punctuation">:</span> SingleQueryResult <span class="token operator">=</span> <span class="token keyword">await</span> transaction<span class="token punctuation">.</span>fetch_row<span class="token punctuation">(</span>
            <span class="token string">"SELECT username FROM users WHERE id = $1"</span><span class="token punctuation">,</span>
            <span class="token punctuation">[</span><span class="token number">100</span><span class="token punctuation">]</span><span class="token punctuation">,</span>
        <span class="token punctuation">)</span>
    dict_result<span class="token punctuation">:</span> Dict<span class="token punctuation">[</span>Any<span class="token punctuation">,</span> Any<span class="token punctuation">]</span> <span class="token operator">=</span> query_result<span class="token punctuation">.</span>result<span class="token punctuation">(</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="fetch-val" tabindex="-1"><a class="header-anchor" href="#fetch-val"><span>Fetch Val</span></a></h3>
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
    <span class="token keyword">async</span> <span class="token keyword">with</span> connection<span class="token punctuation">.</span>transaction<span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token keyword">as</span> transaction<span class="token punctuation">:</span>
        <span class="token comment"># this will be an int value</span>
        query_result_value <span class="token operator">=</span> <span class="token keyword">await</span> transaction<span class="token punctuation">.</span>fetch_row<span class="token punctuation">(</span>
            <span class="token string">"SELECT COUNT(*) FROM users WHERE id > $1"</span><span class="token punctuation">,</span>
            <span class="token punctuation">[</span><span class="token number">100</span><span class="token punctuation">]</span><span class="token punctuation">,</span>
        <span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="pipeline" tabindex="-1"><a class="header-anchor" href="#pipeline"><span>Pipeline</span></a></h3>
<h4 id="parameters-5" tabindex="-1"><a class="header-anchor" href="#parameters-5"><span>Parameters</span></a></h4>
<ul>
<li><code v-pre>queries</code>: list of tuple. It must have structure like</li>
<li><code v-pre>prepared</code>: should the querystring/querystrings be prepared before the request. By default any querystrings will be prepared.</li>
</ul>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code>queries <span class="token operator">=</span> <span class="token punctuation">[</span>
    <span class="token punctuation">(</span><span class="token string">"SELECT * FROM users WHERE name = $1"</span><span class="token punctuation">,</span> <span class="token punctuation">[</span><span class="token string">"some_name"</span><span class="token punctuation">]</span><span class="token punctuation">)</span><span class="token punctuation">,</span>
    <span class="token punctuation">(</span><span class="token string">"SELECT 1"</span><span class="token punctuation">,</span> <span class="token boolean">None</span><span class="token punctuation">)</span><span class="token punctuation">,</span>
<span class="token punctuation">]</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><ul>
<li><code v-pre>prepared</code>: Prepare statements before execution or not.</li>
</ul>
<p>Execute queries in pipeline.
Pipelining can improve performance in use cases in which multiple,
independent queries need to be executed.
In a traditional workflow,
each query is sent to the server after the previous query completes.
In contrast, pipelining allows the client to send all of the
queries to the server up front, minimizing time spent
by one side waiting for the other to finish sending data:</p>
<div class="language-text line-numbers-mode" data-ext="text" data-title="text"><pre v-pre class="language-text"><code>            Sequential                               Pipelined
| Client         | Server          |    | Client         | Server          |
|----------------|-----------------|    |----------------|-----------------|
| send query 1   |                 |    | send query 1   |                 |
|                | process query 1 |    | send query 2   | process query 1 |
| receive rows 1 |                 |    | send query 3   | process query 2 |
| send query 2   |                 |    | receive rows 1 | process query 3 |
|                | process query 2 |    | receive rows 2 |                 |
| receive rows 2 |                 |    | receive rows 3 |                 |
| send query 3   |                 |
|                | process query 3 |
| receive rows 3 |                 |
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><p><a href="https://docs.rs/tokio-postgres/latest/tokio_postgres/#pipelining" target="_blank" rel="noopener noreferrer">Read more!<ExternalLinkIcon/></a></p>
<p>Full example:</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">import</span> asyncio

<span class="token keyword">from</span> psqlpy <span class="token keyword">import</span> ConnectionPool<span class="token punctuation">,</span> QueryResult


<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    db_pool <span class="token operator">=</span> ConnectionPool<span class="token punctuation">(</span><span class="token punctuation">)</span>
    <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>startup<span class="token punctuation">(</span><span class="token punctuation">)</span>

    connection <span class="token operator">=</span> <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>connection<span class="token punctuation">(</span><span class="token punctuation">)</span>
    transaction <span class="token operator">=</span> connection<span class="token punctuation">.</span>transaction<span class="token punctuation">(</span><span class="token punctuation">)</span>

    results<span class="token punctuation">:</span> <span class="token builtin">list</span><span class="token punctuation">[</span>QueryResult<span class="token punctuation">]</span> <span class="token operator">=</span> <span class="token keyword">await</span> transaction<span class="token punctuation">.</span>pipeline<span class="token punctuation">(</span>
        queries<span class="token operator">=</span><span class="token punctuation">[</span>
            <span class="token punctuation">(</span>
                <span class="token string">"SELECT username FROM users WHERE id = $1"</span><span class="token punctuation">,</span>
                <span class="token punctuation">[</span><span class="token number">100</span><span class="token punctuation">]</span><span class="token punctuation">,</span>
            <span class="token punctuation">)</span><span class="token punctuation">,</span>
            <span class="token punctuation">(</span>
                <span class="token string">"SELECT some_data FROM profiles"</span><span class="token punctuation">,</span>
                <span class="token boolean">None</span><span class="token punctuation">,</span>
            <span class="token punctuation">)</span><span class="token punctuation">,</span>
            <span class="token punctuation">(</span>
                <span class="token string">"INSERT INTO users (username, id) VALUES ($1, $2)"</span><span class="token punctuation">,</span>
                <span class="token punctuation">[</span><span class="token string">"PSQLPy"</span><span class="token punctuation">,</span> <span class="token number">1</span><span class="token punctuation">]</span><span class="token punctuation">,</span>
            <span class="token punctuation">)</span><span class="token punctuation">,</span>
        <span class="token punctuation">]</span>
    <span class="token punctuation">)</span>

</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="create-savepoint" tabindex="-1"><a class="header-anchor" href="#create-savepoint"><span>Create Savepoint</span></a></h3>
<h4 id="parameters-6" tabindex="-1"><a class="header-anchor" href="#parameters-6"><span>Parameters:</span></a></h4>
<ul>
<li><code v-pre>savepoint_name</code>: name of the new savepoint.</li>
</ul>
<p>Savepoint creation. <a href="https://www.postgresql.org/docs/current/sql-savepoint.html" target="_blank" rel="noopener noreferrer">PostgreSQL docs<ExternalLinkIcon/></a></p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span>
    <span class="token keyword">await</span> transaction<span class="token punctuation">.</span>create_savepoint<span class="token punctuation">(</span><span class="token string">"my_savepoint"</span><span class="token punctuation">)</span>
    <span class="token keyword">await</span> transaction<span class="token punctuation">.</span>execute<span class="token punctuation">(</span><span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">)</span>
    <span class="token keyword">await</span> transaction<span class="token punctuation">.</span>rollback_savepoint<span class="token punctuation">(</span><span class="token string">"my_savepoint"</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="rollback" tabindex="-1"><a class="header-anchor" href="#rollback"><span>Rollback</span></a></h3>
<p>Rollback the whole transaction. <a href="https://www.postgresql.org/docs/current/sql-rollback.html" target="_blank" rel="noopener noreferrer">PostgreSQL docs<ExternalLinkIcon/></a></p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span>
    <span class="token keyword">await</span> transaction<span class="token punctuation">.</span>execute<span class="token punctuation">(</span><span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">)</span>
    <span class="token keyword">await</span> transaction<span class="token punctuation">.</span>rollback<span class="token punctuation">(</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="rollback-savepoint" tabindex="-1"><a class="header-anchor" href="#rollback-savepoint"><span>Rollback Savepoint</span></a></h3>
<h4 id="parameters-7" tabindex="-1"><a class="header-anchor" href="#parameters-7"><span>Parameters:</span></a></h4>
<ul>
<li><code v-pre>savepoint_name</code>: name of the new savepoint.</li>
</ul>
<p>Rollback to the specified savepoint. <a href="https://www.postgresql.org/docs/current/sql-savepoint.html" target="_blank" rel="noopener noreferrer">PostgreSQL docs<ExternalLinkIcon/></a></p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span>
    transaction <span class="token operator">=</span> connection<span class="token punctuation">.</span>transaction<span class="token punctuation">(</span><span class="token punctuation">)</span>

    <span class="token keyword">await</span> transaction<span class="token punctuation">.</span>create_savepoint<span class="token punctuation">(</span><span class="token string">"my_savepoint"</span><span class="token punctuation">)</span>
    <span class="token keyword">await</span> transaction<span class="token punctuation">.</span>execute<span class="token punctuation">(</span><span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">)</span>
    <span class="token keyword">await</span> transaction<span class="token punctuation">.</span>rollback_savepoint<span class="token punctuation">(</span><span class="token string">"my_savepoint"</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="release-savepoint" tabindex="-1"><a class="header-anchor" href="#release-savepoint"><span>Release Savepoint</span></a></h3>
<h4 id="parameters-8" tabindex="-1"><a class="header-anchor" href="#parameters-8"><span>Parameters:</span></a></h4>
<ul>
<li><code v-pre>savepoint_name</code>: name of the new savepoint.</li>
</ul>
<p>Release savepoint. <a href="https://www.postgresql.org/docs/current/sql-savepoint.html" target="_blank" rel="noopener noreferrer">PostgreSQL docs<ExternalLinkIcon/></a></p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span>
    connection <span class="token operator">=</span> <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>connection<span class="token punctuation">(</span><span class="token punctuation">)</span>
    transaction <span class="token operator">=</span> connection<span class="token punctuation">.</span>transaction<span class="token punctuation">(</span><span class="token punctuation">)</span>

    <span class="token keyword">await</span> transaction<span class="token punctuation">.</span>create_savepoint<span class="token punctuation">(</span><span class="token string">"my_savepoint"</span><span class="token punctuation">)</span>
    <span class="token keyword">await</span> transaction<span class="token punctuation">.</span>release_savepoint
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="cursor" tabindex="-1"><a class="header-anchor" href="#cursor"><span>Cursor</span></a></h3>
<h4 id="parameters-9" tabindex="-1"><a class="header-anchor" href="#parameters-9"><span>Parameters</span></a></h4>
<ul>
<li><code v-pre>querystring</code>: Statement string.</li>
<li><code v-pre>parameters</code>: List of list of parameters for the statement string.</li>
<li><code v-pre>fetch_number</code>: rewrite default fetch_number. Default is 10.</li>
<li><code v-pre>scroll</code>: make cursor scrollable or not. Default is like in <code v-pre>PostgreSQL</code>.</li>
<li><code v-pre>prepared</code>: prepare querystring or not.</li>
</ul>
<p>From <code v-pre>Transaction</code> you can create new <code v-pre>Cursor</code> object which represents cursor in the <code v-pre>PostgreSQL</code>. <a href="https://www.postgresql.org/docs/current/plpgsql-cursors.html" target="_blank" rel="noopener noreferrer">PostgreSQL Docs<ExternalLinkIcon/></a></p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span>
    transaction <span class="token operator">=</span> <span class="token keyword">await</span> connection<span class="token punctuation">.</span>transaction<span class="token punctuation">(</span><span class="token punctuation">)</span>

    cursor <span class="token operator">=</span> transaction<span class="token punctuation">.</span>cursor<span class="token punctuation">(</span>
        querystring<span class="token operator">=</span><span class="token string">"SELECT * FROM users WHERE username = $1"</span><span class="token punctuation">,</span>
        parameters<span class="token operator">=</span><span class="token punctuation">[</span><span class="token string">"Some_Username"</span><span class="token punctuation">]</span><span class="token punctuation">,</span>
        fetch_number<span class="token operator">=</span><span class="token number">5</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>
    <span class="token keyword">await</span> cursor<span class="token punctuation">.</span>start<span class="token punctuation">(</span><span class="token punctuation">)</span>

    <span class="token keyword">async</span> <span class="token keyword">for</span> fetched_result <span class="token keyword">in</span> cursor<span class="token punctuation">:</span>
        dict_result<span class="token punctuation">:</span> List<span class="token punctuation">[</span>Dict<span class="token punctuation">[</span>Any<span class="token punctuation">,</span> Any<span class="token punctuation">]</span><span class="token punctuation">]</span> <span class="token operator">=</span> fetched_result<span class="token punctuation">.</span>result<span class="token punctuation">(</span><span class="token punctuation">)</span>
        <span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span> <span class="token comment"># do something with the result.</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div></div></template>


