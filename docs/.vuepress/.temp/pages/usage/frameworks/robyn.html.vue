<template><div><p>There is the default example for <code v-pre>Robyn</code> framework.</p>
<p>We strongly recommend to use the following example as a standard way to use <code v-pre>PSQLPy</code> with <code v-pre>Robyn</code> framework.</p>
<h2 id="complete-example" tabindex="-1"><a class="header-anchor" href="#complete-example"><span>Complete example</span></a></h2>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token comment"># Start example</span>
<span class="token keyword">from</span> __future__ <span class="token keyword">import</span> annotations

<span class="token keyword">import</span> asyncio
<span class="token keyword">from</span> typing <span class="token keyword">import</span> Any

<span class="token keyword">from</span> psqlpy <span class="token keyword">import</span> ConnectionPool
<span class="token keyword">from</span> robyn <span class="token keyword">import</span> Request<span class="token punctuation">,</span> Robyn

db_pool <span class="token operator">=</span> ConnectionPool<span class="token punctuation">(</span>
    dsn<span class="token operator">=</span><span class="token string">"postgres://postgres:postgres@localhost:5432/postgres"</span><span class="token punctuation">,</span>
    max_db_pool_size<span class="token operator">=</span><span class="token number">10</span><span class="token punctuation">,</span>
<span class="token punctuation">)</span>

app <span class="token operator">=</span> Robyn<span class="token punctuation">(</span>__file__<span class="token punctuation">)</span>


<span class="token decorator annotation punctuation">@app<span class="token punctuation">.</span>get</span><span class="token punctuation">(</span><span class="token string">"/"</span><span class="token punctuation">)</span>
<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">pg_pool_example</span><span class="token punctuation">(</span>request<span class="token punctuation">:</span> Request<span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token builtin">list</span><span class="token punctuation">[</span><span class="token builtin">dict</span><span class="token punctuation">[</span>Any<span class="token punctuation">,</span> Any<span class="token punctuation">]</span><span class="token punctuation">]</span><span class="token punctuation">:</span>
    connection <span class="token operator">=</span> <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>connection<span class="token punctuation">(</span><span class="token punctuation">)</span>
    query_result <span class="token operator">=</span> <span class="token keyword">await</span> connection<span class="token punctuation">.</span>execute<span class="token punctuation">(</span>
        <span class="token string">"SELECT * FROM users"</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>
    <span class="token keyword">return</span> query_result<span class="token punctuation">.</span>result<span class="token punctuation">(</span><span class="token punctuation">)</span>


<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token keyword">try</span><span class="token punctuation">:</span>
        app<span class="token punctuation">.</span>start<span class="token punctuation">(</span>host<span class="token operator">=</span><span class="token string">"127.0.0.1"</span><span class="token punctuation">,</span> port<span class="token operator">=</span><span class="token number">8000</span><span class="token punctuation">)</span>
    <span class="token keyword">finally</span><span class="token punctuation">:</span>
        db_pool<span class="token punctuation">.</span>close<span class="token punctuation">(</span><span class="token punctuation">)</span>


<span class="token keyword">if</span> __name__ <span class="token operator">==</span> <span class="token string">"__main__"</span><span class="token punctuation">:</span>
    asyncio<span class="token punctuation">.</span>run<span class="token punctuation">(</span>main<span class="token punctuation">(</span><span class="token punctuation">)</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div></div></template>


