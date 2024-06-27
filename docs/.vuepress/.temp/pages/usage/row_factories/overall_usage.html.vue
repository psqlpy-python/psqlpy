<template><div><p><code v-pre>row_factory</code> must be used when you want to process result from Database in a custom way and return something different from dictionary.</p>
<p><code v-pre>row_factory</code> requires a function that accepts parameter <code v-pre>Dict[str, typing.Any]</code> and can return anything you want.</p>
<div class="hint-container tip">
<p class="hint-container-title">Tips</p>
<p><code v-pre>row_factory</code> can be a function or a class with <code v-pre>__call__</code> method which returns target converted instance.</p>
</div>
<h3 id="example" tabindex="-1"><a class="header-anchor" href="#example"><span>Example:</span></a></h3>
<p>We create custom class and function with this class as a parameter and return function which will be used in processing row from database.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token decorator annotation punctuation">@dataclass</span>
<span class="token keyword">class</span> <span class="token class-name">ValidationTestModel</span><span class="token punctuation">:</span>
    <span class="token builtin">id</span><span class="token punctuation">:</span> <span class="token builtin">int</span>
    name<span class="token punctuation">:</span> <span class="token builtin">str</span>

<span class="token keyword">def</span> <span class="token function">to_class</span><span class="token punctuation">(</span>
    class_<span class="token punctuation">:</span> Type<span class="token punctuation">[</span>ValidationTestModel<span class="token punctuation">]</span><span class="token punctuation">,</span>
<span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> Callable<span class="token punctuation">[</span><span class="token punctuation">[</span>Dict<span class="token punctuation">[</span><span class="token builtin">str</span><span class="token punctuation">,</span> Any<span class="token punctuation">]</span><span class="token punctuation">]</span><span class="token punctuation">,</span> ValidationTestModel<span class="token punctuation">]</span><span class="token punctuation">:</span>
    <span class="token keyword">def</span> <span class="token function">to_class_inner</span><span class="token punctuation">(</span>row<span class="token punctuation">:</span> Dict<span class="token punctuation">[</span><span class="token builtin">str</span><span class="token punctuation">,</span> Any<span class="token punctuation">]</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> ValidationTestModel<span class="token punctuation">:</span>
        <span class="token keyword">return</span> class_<span class="token punctuation">(</span><span class="token operator">**</span>row<span class="token punctuation">)</span>

    <span class="token keyword">return</span> to_class_inner

<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    conn_result <span class="token operator">=</span> <span class="token keyword">await</span> psql_pool<span class="token punctuation">.</span>execute<span class="token punctuation">(</span>
        querystring<span class="token operator">=</span><span class="token string-interpolation"><span class="token string">f"SELECT * FROM </span><span class="token interpolation"><span class="token punctuation">{</span>table_name<span class="token punctuation">}</span></span><span class="token string">"</span></span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>
    class_res <span class="token operator">=</span> conn_result<span class="token punctuation">.</span>row_factory<span class="token punctuation">(</span>row_factory<span class="token operator">=</span>to_class<span class="token punctuation">(</span>ValidationTestModel<span class="token punctuation">)</span><span class="token punctuation">)</span>

    <span class="token keyword">assert</span> <span class="token builtin">isinstance</span><span class="token punctuation">(</span>class_res<span class="token punctuation">[</span><span class="token number">0</span><span class="token punctuation">]</span><span class="token punctuation">,</span> ValidationTestModel<span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div></div></template>


