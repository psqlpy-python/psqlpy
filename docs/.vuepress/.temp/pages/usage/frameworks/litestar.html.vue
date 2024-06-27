<template><div><p>There is the default example for <code v-pre>Litestar</code> framework.</p>
<p>We strongly recommend to use the following example as a standard way to use <code v-pre>PSQLPy</code> with <code v-pre>Litestar</code> framework.</p>
<h2 id="complete-example" tabindex="-1"><a class="header-anchor" href="#complete-example"><span>Complete example</span></a></h2>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token comment"># Start example</span>
<span class="token keyword">from</span> __future__ <span class="token keyword">import</span> annotations

<span class="token keyword">from</span> typing <span class="token keyword">import</span> Any<span class="token punctuation">,</span> cast

<span class="token keyword">import</span> uvicorn
<span class="token keyword">from</span> litestar <span class="token keyword">import</span> Litestar<span class="token punctuation">,</span> Request<span class="token punctuation">,</span> get
<span class="token keyword">from</span> psqlpy <span class="token keyword">import</span> ConnectionPool


<span class="token keyword">def</span> <span class="token function">start_db_pool</span><span class="token punctuation">(</span>app<span class="token punctuation">:</span> Litestar<span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> ConnectionPool<span class="token punctuation">:</span>
    <span class="token triple-quoted-string string">"""Return the db pool.

    If it doesn't exist, creates it and saves it in on the application state object
    """</span>
    <span class="token keyword">if</span> <span class="token keyword">not</span> <span class="token builtin">getattr</span><span class="token punctuation">(</span>app<span class="token punctuation">.</span>state<span class="token punctuation">,</span> <span class="token string">"db_pool"</span><span class="token punctuation">,</span> <span class="token boolean">None</span><span class="token punctuation">)</span><span class="token punctuation">:</span>
        app<span class="token punctuation">.</span>state<span class="token punctuation">.</span>db_pool <span class="token operator">=</span> ConnectionPool<span class="token punctuation">(</span>
            dsn<span class="token operator">=</span><span class="token string">"postgres://postgres:postgres@localhost:5432/postgres"</span><span class="token punctuation">,</span>
            max_db_pool_size<span class="token operator">=</span><span class="token number">10</span><span class="token punctuation">,</span>
        <span class="token punctuation">)</span>

    <span class="token keyword">return</span> cast<span class="token punctuation">(</span><span class="token string">"ConnectionPool"</span><span class="token punctuation">,</span> app<span class="token punctuation">.</span>state<span class="token punctuation">.</span>db_pool<span class="token punctuation">)</span>


<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">stop_db_pool</span><span class="token punctuation">(</span>app<span class="token punctuation">:</span> Litestar<span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token triple-quoted-string string">"""Close database connection pool."""</span>
    <span class="token keyword">if</span> <span class="token builtin">getattr</span><span class="token punctuation">(</span>app<span class="token punctuation">.</span>state<span class="token punctuation">,</span> <span class="token string">"engine"</span><span class="token punctuation">,</span> <span class="token boolean">None</span><span class="token punctuation">)</span><span class="token punctuation">:</span>
        db_pool <span class="token operator">=</span> cast<span class="token punctuation">(</span>ConnectionPool<span class="token punctuation">,</span> app<span class="token punctuation">.</span>state<span class="token punctuation">.</span>db_pool<span class="token punctuation">)</span>
        db_pool<span class="token punctuation">.</span>close<span class="token punctuation">(</span><span class="token punctuation">)</span>


<span class="token decorator annotation punctuation">@get</span><span class="token punctuation">(</span><span class="token string">"/"</span><span class="token punctuation">)</span>
<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">pg_pool_example</span><span class="token punctuation">(</span>request<span class="token punctuation">:</span> Request<span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token builtin">list</span><span class="token punctuation">[</span><span class="token builtin">dict</span><span class="token punctuation">[</span>Any<span class="token punctuation">,</span> Any<span class="token punctuation">]</span><span class="token punctuation">]</span><span class="token punctuation">:</span>
    db_pool <span class="token operator">=</span> cast<span class="token punctuation">(</span>ConnectionPool<span class="token punctuation">,</span> request<span class="token punctuation">.</span>app<span class="token punctuation">.</span>state<span class="token punctuation">.</span>db_pool<span class="token punctuation">)</span>
    connection <span class="token operator">=</span> <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>connection<span class="token punctuation">(</span><span class="token punctuation">)</span>
    query_result <span class="token operator">=</span> <span class="token keyword">await</span> connection<span class="token punctuation">.</span>execute<span class="token punctuation">(</span>
        <span class="token string">"SELECT * FROM users"</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>
    <span class="token keyword">return</span> query_result<span class="token punctuation">.</span>result<span class="token punctuation">(</span><span class="token punctuation">)</span>


app <span class="token operator">=</span> Litestar<span class="token punctuation">(</span>
    <span class="token punctuation">[</span>pg_pool_example<span class="token punctuation">]</span><span class="token punctuation">,</span>
    on_startup<span class="token operator">=</span><span class="token punctuation">[</span>start_db_pool<span class="token punctuation">]</span><span class="token punctuation">,</span>
    on_shutdown<span class="token operator">=</span><span class="token punctuation">[</span>stop_db_pool<span class="token punctuation">]</span><span class="token punctuation">,</span>
<span class="token punctuation">)</span>


<span class="token keyword">if</span> __name__ <span class="token operator">==</span> <span class="token string">"__main__"</span><span class="token punctuation">:</span>
    uvicorn<span class="token punctuation">.</span>run<span class="token punctuation">(</span>
        <span class="token string">"start_example:app"</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>

</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div></div></template>


