<template><div><p><code v-pre>Results</code> are objects that driver returns to python with some data inside.</p>
<p>Currently there are two results:</p>
<ul>
<li><code v-pre>QueryResult</code> - for result with multiple rows</li>
<li><code v-pre>SingleQueryResult</code> - for result with exactly one row</li>
</ul>
<h2 id="queryresult-methods" tabindex="-1"><a class="header-anchor" href="#queryresult-methods"><span>QueryResult methods</span></a></h2>
<h3 id="result" tabindex="-1"><a class="header-anchor" href="#result"><span>Result</span></a></h3>
<h4 id="parameters" tabindex="-1"><a class="header-anchor" href="#parameters"><span>Parameters</span></a></h4>
<ul>
<li><code v-pre>custom_decoders</code>: custom decoders for unsupported types. <RouteLink to="/usage/types/advanced_type_usage.html">Read more</RouteLink></li>
</ul>
<p>Get the result as a list of dicts</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    db_pool <span class="token operator">=</span> ConnectionPool<span class="token punctuation">(</span><span class="token punctuation">)</span>
    connection <span class="token operator">=</span> <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>connection<span class="token punctuation">(</span><span class="token punctuation">)</span>
    query_result<span class="token punctuation">:</span> QueryResult <span class="token operator">=</span> <span class="token keyword">await</span> connection<span class="token punctuation">.</span>execute<span class="token punctuation">(</span>
        <span class="token string">"SELECT username FROM users"</span><span class="token punctuation">,</span>
        <span class="token punctuation">[</span><span class="token punctuation">]</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>

    result<span class="token punctuation">:</span> List<span class="token punctuation">[</span>Dict<span class="token punctuation">[</span><span class="token builtin">str</span><span class="token punctuation">,</span> Any<span class="token punctuation">]</span><span class="token punctuation">]</span> <span class="token operator">=</span> query_result<span class="token punctuation">.</span>result<span class="token punctuation">(</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="as-class" tabindex="-1"><a class="header-anchor" href="#as-class"><span>As class</span></a></h3>
<h4 id="parameters-1" tabindex="-1"><a class="header-anchor" href="#parameters-1"><span>Parameters</span></a></h4>
<ul>
<li><code v-pre>as_class</code>: Custom class from Python.</li>
<li><code v-pre>custom_decoders</code>: custom decoders for unsupported types. <RouteLink to="/usage/types/advanced_type_usage.html">Read more</RouteLink></li>
</ul>
<p>Get the result as a list of passed class instances.
Passed class can easily be either pydantic or msgspec model.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">class</span> <span class="token class-name">ExampleOfAsClass</span><span class="token punctuation">:</span>
    <span class="token keyword">def</span> <span class="token function">__init__</span><span class="token punctuation">(</span>self<span class="token punctuation">,</span> username<span class="token punctuation">:</span> <span class="token builtin">str</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
        self<span class="token punctuation">.</span>username <span class="token operator">=</span> username


<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    db_pool <span class="token operator">=</span> ConnectionPool<span class="token punctuation">(</span><span class="token punctuation">)</span>
    connection <span class="token operator">=</span> <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>connection<span class="token punctuation">(</span><span class="token punctuation">)</span>
    query_result<span class="token punctuation">:</span> QueryResult <span class="token operator">=</span> <span class="token keyword">await</span> connection<span class="token punctuation">.</span>execute<span class="token punctuation">(</span>
        <span class="token string">"SELECT username FROM users"</span><span class="token punctuation">,</span>
        <span class="token punctuation">[</span><span class="token punctuation">]</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>

    class_results<span class="token punctuation">:</span> List<span class="token punctuation">[</span>ExampleOfAsClass<span class="token punctuation">]</span> <span class="token operator">=</span> query_result<span class="token punctuation">.</span>as_class<span class="token punctuation">(</span>
        as_class<span class="token operator">=</span>ExampleOfAsClass<span class="token punctuation">,</span>
    <span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="row-factory" tabindex="-1"><a class="header-anchor" href="#row-factory"><span>Row Factory</span></a></h3>
<h4 id="parameters-2" tabindex="-1"><a class="header-anchor" href="#parameters-2"><span>Parameters</span></a></h4>
<ul>
<li><code v-pre>row_factory</code>: custom callable object.</li>
<li><code v-pre>custom_decoders</code>: custom decoders for unsupported types. <RouteLink to="/usage/types/advanced_type_usage.html">Read more</RouteLink></li>
</ul>
<p><RouteLink to="/usage/row_factories/overall_usage.html">Read more</RouteLink></p>
<h2 id="singlequeryresult-methods" tabindex="-1"><a class="header-anchor" href="#singlequeryresult-methods"><span>SingleQueryResult methods</span></a></h2>
<h3 id="result-1" tabindex="-1"><a class="header-anchor" href="#result-1"><span>Result</span></a></h3>
<h4 id="parameters-3" tabindex="-1"><a class="header-anchor" href="#parameters-3"><span>Parameters</span></a></h4>
<ul>
<li><code v-pre>custom_decoders</code>: custom decoders for unsupported types. <RouteLink to="/usage/types/advanced_type_usage.html">Read more</RouteLink></li>
</ul>
<p>Get the result as a dict</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    db_pool <span class="token operator">=</span> ConnectionPool<span class="token punctuation">(</span><span class="token punctuation">)</span>
    connection <span class="token operator">=</span> <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>connection<span class="token punctuation">(</span><span class="token punctuation">)</span>
    query_result<span class="token punctuation">:</span> SingleQueryResult <span class="token operator">=</span> <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>fetch_row<span class="token punctuation">(</span>
        <span class="token string">"SELECT username FROM users WHERE id = $1"</span><span class="token punctuation">,</span>
        <span class="token punctuation">[</span><span class="token number">100</span><span class="token punctuation">]</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>

    result<span class="token punctuation">:</span> Dict<span class="token punctuation">[</span><span class="token builtin">str</span><span class="token punctuation">,</span> Any<span class="token punctuation">]</span> <span class="token operator">=</span> query_result<span class="token punctuation">.</span>result<span class="token punctuation">(</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="as-class-1" tabindex="-1"><a class="header-anchor" href="#as-class-1"><span>As class</span></a></h3>
<h4 id="parameters-4" tabindex="-1"><a class="header-anchor" href="#parameters-4"><span>Parameters</span></a></h4>
<ul>
<li><code v-pre>as_class</code>: Custom class from Python.</li>
<li><code v-pre>custom_decoders</code>: custom decoders for unsupported types. <RouteLink to="/usage/types/advanced_type_usage.html">Read more</RouteLink></li>
</ul>
<p>Get the result as a passed class instance.
Passed class can easily be either pydantic or msgspec model.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">class</span> <span class="token class-name">ExampleOfAsClass</span><span class="token punctuation">:</span>
    <span class="token keyword">def</span> <span class="token function">__init__</span><span class="token punctuation">(</span>self<span class="token punctuation">,</span> username<span class="token punctuation">:</span> <span class="token builtin">str</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
        self<span class="token punctuation">.</span>username <span class="token operator">=</span> username


<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    db_pool <span class="token operator">=</span> ConnectionPool<span class="token punctuation">(</span><span class="token punctuation">)</span>
    connection <span class="token operator">=</span> <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>connection<span class="token punctuation">(</span><span class="token punctuation">)</span>
    query_result<span class="token punctuation">:</span> SingleQueryResult <span class="token operator">=</span> <span class="token keyword">await</span> connection<span class="token punctuation">.</span>fetch_row<span class="token punctuation">(</span>
        <span class="token string">"SELECT username FROM users WHERE id = $1"</span><span class="token punctuation">,</span>
        <span class="token punctuation">[</span><span class="token number">100</span><span class="token punctuation">]</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>
    class_results<span class="token punctuation">:</span> ExampleOfAsClass <span class="token operator">=</span> query_result<span class="token punctuation">.</span>as_class<span class="token punctuation">(</span>
        as_class<span class="token operator">=</span>ExampleOfAsClass<span class="token punctuation">,</span>
    <span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="row-factory-1" tabindex="-1"><a class="header-anchor" href="#row-factory-1"><span>Row Factory</span></a></h3>
<h4 id="parameters-5" tabindex="-1"><a class="header-anchor" href="#parameters-5"><span>Parameters</span></a></h4>
<ul>
<li><code v-pre>row_factory</code>: custom callable object.</li>
<li><code v-pre>custom_decoders</code>: custom decoders for unsupported types. <RouteLink to="/usage/types/advanced_type_usage.html">Read more</RouteLink></li>
</ul>
<p><RouteLink to="/usage/row_factories/overall_usage.html">Read more</RouteLink></p>
</div></template>


