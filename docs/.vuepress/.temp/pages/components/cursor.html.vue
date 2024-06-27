<template><div><p><code v-pre>Cursor</code> objects represents real <code v-pre>Cursor</code> in the <code v-pre>PostgreSQL</code>. <a href="https://www.postgresql.org/docs/current/plpgsql-cursors.html" target="_blank" rel="noopener noreferrer">PostgreSQL docs<ExternalLinkIcon/></a><br>
It can be built only from <code v-pre>Transaction</code>.</p>
<h2 id="cursor-parameters" tabindex="-1"><a class="header-anchor" href="#cursor-parameters"><span>Cursor Parameters</span></a></h2>
<ul>
<li><code v-pre>querystring</code>: specify query for cursor.</li>
<li><code v-pre>parameters</code>: parameters for the querystring. Default <code v-pre>None</code></li>
<li><code v-pre>fetch_number</code>: default fetch number. It is used in <code v-pre>fetch()</code> method and in async iterator. Default 10</li>
<li><code v-pre>scroll</code>: is cursor scrollable or not. Default as in <code v-pre>PostgreSQL</code>.</li>
</ul>
<h2 id="cursor-as-async-iterator" tabindex="-1"><a class="header-anchor" href="#cursor-as-async-iterator"><span>Cursor as async iterator</span></a></h2>
<p>The most common situation is using <code v-pre>Cursor</code> as async iterator.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">from</span> psqlpy <span class="token keyword">import</span> ConnectionPool<span class="token punctuation">,</span> QueryResult


<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    db_pool <span class="token operator">=</span> ConnectionPool<span class="token punctuation">(</span><span class="token punctuation">)</span>


    connection <span class="token operator">=</span> <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>connection<span class="token punctuation">(</span><span class="token punctuation">)</span>
    transaction <span class="token operator">=</span> <span class="token keyword">await</span> connection<span class="token punctuation">.</span>transaction<span class="token punctuation">(</span><span class="token punctuation">)</span>

    <span class="token comment"># Here we fetch 5 results in each iteration.</span>
    <span class="token keyword">async</span> <span class="token keyword">with</span> cursor <span class="token keyword">in</span> transaction<span class="token punctuation">.</span>cursor<span class="token punctuation">(</span>
        querystring<span class="token operator">=</span><span class="token string">"SELECT * FROM users WHERE username = $1"</span><span class="token punctuation">,</span>
        parameters<span class="token operator">=</span><span class="token punctuation">[</span><span class="token string">"Some_Username"</span><span class="token punctuation">]</span><span class="token punctuation">,</span>
        fetch_number<span class="token operator">=</span><span class="token number">5</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span><span class="token punctuation">:</span>
        <span class="token keyword">async</span> <span class="token keyword">for</span> fetched_result <span class="token keyword">in</span> cursor<span class="token punctuation">:</span>
            dict_result<span class="token punctuation">:</span> List<span class="token punctuation">[</span>Dict<span class="token punctuation">[</span>Any<span class="token punctuation">,</span> Any<span class="token punctuation">]</span><span class="token punctuation">]</span> <span class="token operator">=</span> fetched_result<span class="token punctuation">.</span>result<span class="token punctuation">(</span><span class="token punctuation">)</span>
            <span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span> <span class="token comment"># do something with this result.</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h2 id="cursor-methods" tabindex="-1"><a class="header-anchor" href="#cursor-methods"><span>Cursor methods</span></a></h2>
<p>There are a lot of methods to work with cursor.</p>
<h3 id="start" tabindex="-1"><a class="header-anchor" href="#start"><span>Start</span></a></h3>
<p>Declare (create) cursor.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token keyword">await</span> cursor<span class="token punctuation">.</span>start<span class="token punctuation">(</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="close" tabindex="-1"><a class="header-anchor" href="#close"><span>Close</span></a></h3>
<p>Close the cursor</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token keyword">await</span> cursor<span class="token punctuation">.</span>close<span class="token punctuation">(</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="fetch" tabindex="-1"><a class="header-anchor" href="#fetch"><span>Fetch</span></a></h3>
<p>You can fetch next <code v-pre>N</code> records from the cursor.<br>
It's possible to specify <code v-pre>N</code> fetch record with parameter <code v-pre>fetch_number</code>, otherwise will be used <code v-pre>fetch_number</code> from the <code v-pre>Cursor</code> initialization.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    result<span class="token punctuation">:</span> QueryResult <span class="token operator">=</span> <span class="token keyword">await</span> cursor<span class="token punctuation">.</span>fetch<span class="token punctuation">(</span>
        fetch_number<span class="token operator">=</span><span class="token number">100</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="fetch-next" tabindex="-1"><a class="header-anchor" href="#fetch-next"><span>Fetch Next</span></a></h3>
<p>Just fetch next record from the <code v-pre>Cursor</code>.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    result<span class="token punctuation">:</span> QueryResult <span class="token operator">=</span> <span class="token keyword">await</span> cursor<span class="token punctuation">.</span>fetch_next<span class="token punctuation">(</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="fetch-prior" tabindex="-1"><a class="header-anchor" href="#fetch-prior"><span>Fetch Prior</span></a></h3>
<p>Just fetch previous record.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    result<span class="token punctuation">:</span> QueryResult <span class="token operator">=</span> <span class="token keyword">await</span> cursor<span class="token punctuation">.</span>fetch_prior<span class="token punctuation">(</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="fetch-first" tabindex="-1"><a class="header-anchor" href="#fetch-first"><span>Fetch First</span></a></h3>
<p>Just fetch the first record.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    result<span class="token punctuation">:</span> QueryResult <span class="token operator">=</span> <span class="token keyword">await</span> cursor<span class="token punctuation">.</span>fetch_first<span class="token punctuation">(</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="fetch-last" tabindex="-1"><a class="header-anchor" href="#fetch-last"><span>Fetch Last</span></a></h3>
<p>Just fetch the last record.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    result<span class="token punctuation">:</span> QueryResult <span class="token operator">=</span> <span class="token keyword">await</span> cursor<span class="token punctuation">.</span>fetch_last<span class="token punctuation">(</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="fetch-absolute" tabindex="-1"><a class="header-anchor" href="#fetch-absolute"><span>Fetch Absolute</span></a></h3>
<p>Just fetch absolute records.
It has <code v-pre>absolute_number</code> parameter, you must specify it.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    result<span class="token punctuation">:</span> QueryResult <span class="token operator">=</span> <span class="token keyword">await</span> cursor<span class="token punctuation">.</span>fetch_absolute<span class="token punctuation">(</span>
        absolute_number<span class="token operator">=</span><span class="token number">10</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="fetch-relative" tabindex="-1"><a class="header-anchor" href="#fetch-relative"><span>Fetch Relative</span></a></h3>
<p>Just fetch absolute records.
It has <code v-pre>relative_number</code> parameter, you must specify it.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    result<span class="token punctuation">:</span> QueryResult <span class="token operator">=</span> <span class="token keyword">await</span> cursor<span class="token punctuation">.</span>fetch_relative<span class="token punctuation">(</span>
        relative_number<span class="token operator">=</span><span class="token number">10</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="fetch-forward-all" tabindex="-1"><a class="header-anchor" href="#fetch-forward-all"><span>Fetch Forward All</span></a></h3>
<p>Fetch forward all records in the cursor.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    result<span class="token punctuation">:</span> QueryResult <span class="token operator">=</span> <span class="token keyword">await</span> cursor<span class="token punctuation">.</span>fetch_forward_all<span class="token punctuation">(</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="fetch-backward" tabindex="-1"><a class="header-anchor" href="#fetch-backward"><span>Fetch Backward</span></a></h3>
<p>Just backward records.
It has <code v-pre>backward_count</code> parameter, you must specify it.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    result<span class="token punctuation">:</span> QueryResult <span class="token operator">=</span> <span class="token keyword">await</span> cursor<span class="token punctuation">.</span>fetch_backward<span class="token punctuation">(</span>
        backward_count<span class="token operator">=</span><span class="token number">10</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="fetch-backward-all" tabindex="-1"><a class="header-anchor" href="#fetch-backward-all"><span>Fetch Backward All</span></a></h3>
<p>Fetch backward all records in the cursor.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    result<span class="token punctuation">:</span> QueryResult <span class="token operator">=</span> <span class="token keyword">await</span> cursor<span class="token punctuation">.</span>fetch_backward_all<span class="token punctuation">(</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div></div></div></div></template>


