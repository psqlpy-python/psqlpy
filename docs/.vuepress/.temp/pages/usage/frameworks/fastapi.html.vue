<template><div><p>There is the default example for <code v-pre>FastAPI</code> framework.</p>
<h2 id="standard-example" tabindex="-1"><a class="header-anchor" href="#standard-example"><span>Standard example.</span></a></h2>
<p>This code is perfect for situations when your endpoints don't have complex logic
like sending messages over network with some queues (<code v-pre>RabbitMQ</code>, <code v-pre>NATS</code>, <code v-pre>Kafka</code> and etc)
or making long calculations, so a connection won't idle to much.<br>
You need to take this restrictions into account if you don't have external database connection pool
like <code v-pre>PGBouncer</code>.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token comment"># Start example</span>
<span class="token keyword">from</span> contextlib <span class="token keyword">import</span> asynccontextmanager
<span class="token keyword">from</span> typing <span class="token keyword">import</span> Annotated<span class="token punctuation">,</span> AsyncGenerator<span class="token punctuation">,</span> cast
<span class="token keyword">from</span> fastapi <span class="token keyword">import</span> Depends<span class="token punctuation">,</span> FastAPI<span class="token punctuation">,</span> Request
<span class="token keyword">from</span> fastapi<span class="token punctuation">.</span>responses <span class="token keyword">import</span> JSONResponse
<span class="token keyword">from</span> psqlpy <span class="token keyword">import</span> ConnectionPool<span class="token punctuation">,</span> Connection
<span class="token keyword">import</span> uvicorn


<span class="token decorator annotation punctuation">@asynccontextmanager</span>
<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">lifespan</span><span class="token punctuation">(</span>app<span class="token punctuation">:</span> FastAPI<span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> AsyncGenerator<span class="token punctuation">[</span><span class="token boolean">None</span><span class="token punctuation">,</span> <span class="token boolean">None</span><span class="token punctuation">]</span><span class="token punctuation">:</span>
    <span class="token triple-quoted-string string">"""Startup database connection pool and close it on shutdown."""</span>
    db_pool <span class="token operator">=</span> ConnectionPool<span class="token punctuation">(</span>
        dsn<span class="token operator">=</span><span class="token string">"postgres://postgres:postgres@localhost:5432/postgres"</span><span class="token punctuation">,</span>
        max_db_pool_size<span class="token operator">=</span><span class="token number">10</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>
    app<span class="token punctuation">.</span>state<span class="token punctuation">.</span>db_pool <span class="token operator">=</span> db_pool
    <span class="token keyword">yield</span>
    <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>close<span class="token punctuation">(</span><span class="token punctuation">)</span>


app <span class="token operator">=</span> FastAPI<span class="token punctuation">(</span>lifespan<span class="token operator">=</span>lifespan<span class="token punctuation">)</span>


<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">db_connection</span><span class="token punctuation">(</span>request<span class="token punctuation">:</span> Request<span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> Connection<span class="token punctuation">:</span>
    <span class="token triple-quoted-string string">"""Retrieve new connection from connection pool and return it."""</span>
    <span class="token keyword">return</span> <span class="token keyword">await</span> <span class="token punctuation">(</span>cast<span class="token punctuation">(</span>ConnectionPool<span class="token punctuation">,</span> request<span class="token punctuation">.</span>app<span class="token punctuation">.</span>state<span class="token punctuation">.</span>db_pool<span class="token punctuation">)</span><span class="token punctuation">)</span><span class="token punctuation">.</span>connection<span class="token punctuation">(</span><span class="token punctuation">)</span>


<span class="token decorator annotation punctuation">@app<span class="token punctuation">.</span>get</span><span class="token punctuation">(</span><span class="token string">"/"</span><span class="token punctuation">)</span>
<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">pg_pool_example</span><span class="token punctuation">(</span>
    db_connection<span class="token punctuation">:</span> Annotated<span class="token punctuation">[</span>Connection<span class="token punctuation">,</span> Depends<span class="token punctuation">(</span>db_connection<span class="token punctuation">)</span><span class="token punctuation">]</span><span class="token punctuation">,</span>
<span class="token punctuation">)</span><span class="token punctuation">:</span>
    query_result <span class="token operator">=</span> <span class="token keyword">await</span> db_connection<span class="token punctuation">.</span>execute<span class="token punctuation">(</span>
        <span class="token string">"SELECT * FROM users"</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>
    <span class="token keyword">return</span> JSONResponse<span class="token punctuation">(</span>content<span class="token operator">=</span>query_result<span class="token punctuation">.</span>result<span class="token punctuation">(</span><span class="token punctuation">)</span><span class="token punctuation">)</span>


<span class="token keyword">if</span> __name__ <span class="token operator">==</span> <span class="token string">"__main__"</span><span class="token punctuation">:</span>
    uvicorn<span class="token punctuation">.</span>run<span class="token punctuation">(</span>
        <span class="token string">"start_example:app"</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h2 id="advanced-example" tabindex="-1"><a class="header-anchor" href="#advanced-example"><span>Advanced example</span></a></h2>
<p>If you don't have external connection pool like <code v-pre>PGBouncer</code> and your application have a lot of endpoints with a lot of complex logic,
so it's better not to take a connection from a pool at the start of an endpoint execution (don't use <code v-pre>Depends()</code> like in the previous example), because it will be blocked until the end of the endpoint logic.<br>
The main idea is take a connection from a pool only for code parts in which it will be used immediately.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token comment"># Start example</span>
<span class="token keyword">from</span> contextlib <span class="token keyword">import</span> asynccontextmanager
<span class="token keyword">from</span> typing <span class="token keyword">import</span> Annotated<span class="token punctuation">,</span> AsyncGenerator<span class="token punctuation">,</span> cast
<span class="token keyword">from</span> fastapi <span class="token keyword">import</span> Depends<span class="token punctuation">,</span> FastAPI<span class="token punctuation">,</span> Request
<span class="token keyword">from</span> fastapi<span class="token punctuation">.</span>responses <span class="token keyword">import</span> JSONResponse
<span class="token keyword">from</span> psqlpy <span class="token keyword">import</span> ConnectionPool<span class="token punctuation">,</span> Connection
<span class="token keyword">import</span> uvicorn


db_pool <span class="token operator">=</span> ConnectionPool<span class="token punctuation">(</span>
    dsn<span class="token operator">=</span><span class="token string">"postgres://postgres:postgres@localhost:5432/postgres"</span><span class="token punctuation">,</span>
    max_db_pool_size<span class="token operator">=</span><span class="token number">2</span><span class="token punctuation">,</span>
<span class="token punctuation">)</span>


<span class="token decorator annotation punctuation">@asynccontextmanager</span>
<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">lifespan</span><span class="token punctuation">(</span>app<span class="token punctuation">:</span> FastAPI<span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> AsyncGenerator<span class="token punctuation">[</span><span class="token boolean">None</span><span class="token punctuation">,</span> <span class="token boolean">None</span><span class="token punctuation">]</span><span class="token punctuation">:</span>
    <span class="token triple-quoted-string string">"""Startup database connection pool and close it on shutdown."""</span>
    app<span class="token punctuation">.</span>state<span class="token punctuation">.</span>db_pool <span class="token operator">=</span> db_pool
    <span class="token keyword">yield</span>
    db_pool<span class="token punctuation">.</span>close<span class="token punctuation">(</span><span class="token punctuation">)</span>


app <span class="token operator">=</span> FastAPI<span class="token punctuation">(</span>lifespan<span class="token operator">=</span>lifespan<span class="token punctuation">)</span>


<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">some_long_func</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token comment"># Some very long execution.</span>
    <span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span>


<span class="token decorator annotation punctuation">@app<span class="token punctuation">.</span>get</span><span class="token punctuation">(</span><span class="token string">"/"</span><span class="token punctuation">)</span>
<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">pg_pool_example</span><span class="token punctuation">(</span><span class="token punctuation">)</span><span class="token punctuation">:</span>
    <span class="token keyword">await</span> some_long_func<span class="token punctuation">(</span><span class="token punctuation">)</span>
    db_connection <span class="token operator">=</span> <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>connection<span class="token punctuation">(</span><span class="token punctuation">)</span>
    query_result <span class="token operator">=</span> <span class="token keyword">await</span> db_connection<span class="token punctuation">.</span>execute<span class="token punctuation">(</span>
        <span class="token string">"SELECT * FROM users"</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>
    <span class="token keyword">return</span> JSONResponse<span class="token punctuation">(</span>content<span class="token operator">=</span>query_result<span class="token punctuation">.</span>result<span class="token punctuation">(</span><span class="token punctuation">)</span><span class="token punctuation">)</span>


<span class="token keyword">if</span> __name__ <span class="token operator">==</span> <span class="token string">"__main__"</span><span class="token punctuation">:</span>
    uvicorn<span class="token punctuation">.</span>run<span class="token punctuation">(</span>
        <span class="token string">"start_example:app"</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div></div></template>


