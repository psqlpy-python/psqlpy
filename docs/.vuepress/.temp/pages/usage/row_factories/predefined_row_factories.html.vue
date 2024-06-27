<template><div><p>We have predefined row_factories for fast usage.</p>
<h3 id="tuple-row" tabindex="-1"><a class="header-anchor" href="#tuple-row"><span>tuple_row</span></a></h3>
<p>Instead of dict you can return tuple as a result.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">from</span> psqlpy<span class="token punctuation">.</span>row_factories <span class="token keyword">import</span> tuple_row

<span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span>


<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    conn_result <span class="token operator">=</span> <span class="token keyword">await</span> psql_pool<span class="token punctuation">.</span>execute<span class="token punctuation">(</span>
        querystring<span class="token operator">=</span><span class="token string-interpolation"><span class="token string">f"SELECT * FROM </span><span class="token interpolation"><span class="token punctuation">{</span>table_name<span class="token punctuation">}</span></span><span class="token string">"</span></span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>
    tuple_res <span class="token operator">=</span> conn_result<span class="token punctuation">.</span>row_factory<span class="token punctuation">(</span>row_factory<span class="token operator">=</span>tuple_row<span class="token punctuation">)</span>

    <span class="token keyword">assert</span> <span class="token builtin">isinstance</span><span class="token punctuation">(</span>tuple_res<span class="token punctuation">[</span><span class="token number">0</span><span class="token punctuation">]</span><span class="token punctuation">,</span> <span class="token builtin">tuple</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h3 id="class-row" tabindex="-1"><a class="header-anchor" href="#class-row"><span>class_row</span></a></h3>
<p>You can build class from database result.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">from</span> dataclasses <span class="token keyword">import</span> dataclass

<span class="token keyword">from</span> psqlpy<span class="token punctuation">.</span>row_factories <span class="token keyword">import</span> class_row

<span class="token punctuation">.</span><span class="token punctuation">.</span><span class="token punctuation">.</span>


<span class="token decorator annotation punctuation">@dataclass</span>
<span class="token keyword">class</span> <span class="token class-name">ValidationTestModel</span><span class="token punctuation">:</span>
    <span class="token builtin">id</span><span class="token punctuation">:</span> <span class="token builtin">int</span>
    name<span class="token punctuation">:</span> <span class="token builtin">str</span>


<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    conn_result <span class="token operator">=</span> <span class="token keyword">await</span> psql_pool<span class="token punctuation">.</span>execute<span class="token punctuation">(</span>
        querystring<span class="token operator">=</span><span class="token string-interpolation"><span class="token string">f"SELECT * FROM </span><span class="token interpolation"><span class="token punctuation">{</span>table_name<span class="token punctuation">}</span></span><span class="token string">"</span></span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>
    tuple_res <span class="token operator">=</span> conn_result<span class="token punctuation">.</span>row_factory<span class="token punctuation">(</span>row_factory<span class="token operator">=</span>class_row<span class="token punctuation">(</span>ValidationTestModel<span class="token punctuation">)</span><span class="token punctuation">)</span>

    <span class="token keyword">assert</span> <span class="token builtin">isinstance</span><span class="token punctuation">(</span>tuple_res<span class="token punctuation">[</span><span class="token number">0</span><span class="token punctuation">]</span><span class="token punctuation">,</span> ValidationTestModel<span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div></div></template>


