<template><div><p>There is the default example for <code v-pre>AioHTTP</code> framework.</p>
<p>We strongly recommend to use the following example as a standard way to use <code v-pre>PSQLPy</code> with <code v-pre>AioHTTP</code> framework.</p>
<h2 id="complete-example" tabindex="-1"><a class="header-anchor" href="#complete-example"><span>Complete example</span></a></h2>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token comment"># Start example</span>
<span class="token keyword">import</span> asyncio
<span class="token keyword">from</span> typing <span class="token keyword">import</span> cast
<span class="token keyword">from</span> aiohttp <span class="token keyword">import</span> web
<span class="token keyword">from</span> psqlpy <span class="token keyword">import</span> ConnectionPool


<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">start_db_pool</span><span class="token punctuation">(</span>app<span class="token punctuation">:</span> web<span class="token punctuation">.</span>Application<span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token triple-quoted-string string">"""Initialize database connection pool."""</span>
    db_pool <span class="token operator">=</span> ConnectionPool<span class="token punctuation">(</span>
        dsn<span class="token operator">=</span><span class="token string">"postgres://postgres:postgres@localhost:5432/postgres"</span><span class="token punctuation">,</span>
        max_db_pool_size<span class="token operator">=</span><span class="token number">10</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>

    app<span class="token punctuation">[</span><span class="token string">"db_pool"</span><span class="token punctuation">]</span> <span class="token operator">=</span> db_pool


<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">stop_db_pool</span><span class="token punctuation">(</span>app<span class="token punctuation">:</span> web<span class="token punctuation">.</span>Application<span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token triple-quoted-string string">"""Close database connection pool."""</span>
    db_pool <span class="token operator">=</span> cast<span class="token punctuation">(</span>ConnectionPool<span class="token punctuation">,</span> app<span class="token punctuation">.</span>db_pool<span class="token punctuation">)</span>
    db_pool<span class="token punctuation">.</span>close<span class="token punctuation">(</span><span class="token punctuation">)</span>


<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">pg_pool_example</span><span class="token punctuation">(</span>request<span class="token punctuation">:</span> web<span class="token punctuation">.</span>Request<span class="token punctuation">)</span><span class="token punctuation">:</span>
    db_pool <span class="token operator">=</span> cast<span class="token punctuation">(</span>ConnectionPool<span class="token punctuation">,</span> request<span class="token punctuation">.</span>app<span class="token punctuation">[</span><span class="token string">"db_pool"</span><span class="token punctuation">]</span><span class="token punctuation">)</span>
    connection <span class="token operator">=</span> <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>connection<span class="token punctuation">(</span><span class="token punctuation">)</span>
    <span class="token keyword">await</span> asyncio<span class="token punctuation">.</span>sleep<span class="token punctuation">(</span><span class="token number">10</span><span class="token punctuation">)</span>
    query_result <span class="token operator">=</span> <span class="token keyword">await</span> connection<span class="token punctuation">.</span>execute<span class="token punctuation">(</span>
        <span class="token string">"SELECT * FROM users"</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>
    dict_result <span class="token operator">=</span> query_result<span class="token punctuation">.</span>result<span class="token punctuation">(</span><span class="token punctuation">)</span>
    <span class="token keyword">return</span> web<span class="token punctuation">.</span>json_response<span class="token punctuation">(</span>
        data<span class="token operator">=</span>dict_result<span class="token punctuation">,</span>
    <span class="token punctuation">)</span>


application <span class="token operator">=</span> web<span class="token punctuation">.</span>Application<span class="token punctuation">(</span><span class="token punctuation">)</span>
application<span class="token punctuation">.</span>on_startup<span class="token punctuation">.</span>append<span class="token punctuation">(</span>start_db_pool<span class="token punctuation">)</span>
application<span class="token punctuation">.</span>add_routes<span class="token punctuation">(</span><span class="token punctuation">[</span>web<span class="token punctuation">.</span>get<span class="token punctuation">(</span><span class="token string">'/'</span><span class="token punctuation">,</span> pg_pool_example<span class="token punctuation">)</span><span class="token punctuation">]</span><span class="token punctuation">)</span>


<span class="token keyword">if</span> __name__ <span class="token operator">==</span> <span class="token string">"__main__"</span><span class="token punctuation">:</span>
    web<span class="token punctuation">.</span>run_app<span class="token punctuation">(</span>application<span class="token punctuation">)</span>

</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div></div></template>


