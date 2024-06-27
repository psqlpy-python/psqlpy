<template><div><p>Due to an unavailability to support all possible types in PostgreSQL, we have a way to encode Python types into PostgreSQL ones and decode wise versa.</p>
<p>This section has <code v-pre>Advanced</code> in the name because you'll need to work with raw bytes which can be difficult for some developers.</p>
<h2 id="pass-unsupported-type-into-postgresql" tabindex="-1"><a class="header-anchor" href="#pass-unsupported-type-into-postgresql"><span>Pass unsupported type into PostgreSQL</span></a></h2>
<p>If you are using some type that we don't support and want to insert it into PostgreSQL from PSQLPy, you must use <code v-pre>PyCustomType</code> class.</p>
<p>Let's assume we have table <code v-pre>for_test</code> in the database and <code v-pre>PSQLPy</code> doesn't support (only for demonstration) <code v-pre>VARCHAR</code> type:</p>
<table>
<thead>
<tr>
<th style="text-align:center">database type</th>
<th style="text-align:center">database column name</th>
</tr>
</thead>
<tbody>
<tr>
<td style="text-align:center">VARCHAR</td>
<td style="text-align:center">nickname</td>
</tr>
</tbody>
</table>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">from</span> typing <span class="token keyword">import</span> Final

<span class="token keyword">from</span> psqlpy <span class="token keyword">import</span> ConnectionPool
<span class="token keyword">from</span> psqlpy<span class="token punctuation">.</span>extra_types <span class="token keyword">import</span> PyCustomType


<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token comment"># It uses default connection parameters</span>
    db_pool<span class="token punctuation">:</span> Final <span class="token operator">=</span> ConnectionPool<span class="token punctuation">(</span><span class="token punctuation">)</span>

    <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>execute<span class="token punctuation">(</span>
        <span class="token string">"INSERT INTO for_test (nickname) VALUES ($1)"</span><span class="token punctuation">,</span>
        <span class="token punctuation">[</span>PyCustomType<span class="token punctuation">(</span><span class="token string">b"SomeDataInBytes"</span><span class="token punctuation">)</span><span class="token punctuation">]</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>
    db_pool<span class="token punctuation">.</span>close<span class="token punctuation">(</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><p>Here we pass <code v-pre>PyCustomType</code> into the parameters. It accepts only bytes.</p>
<div class="hint-container important">
<p class="hint-container-title">Important</p>
<p>You must make bytes passed into <code v-pre>PyCustomType</code> readable for <code v-pre>PostgreSQL</code>.<br>
If bytes will be wrong, you will get an exception.</p>
</div>
<h2 id="decode-unsupported-type-from-postgresql" tabindex="-1"><a class="header-anchor" href="#decode-unsupported-type-from-postgresql"><span>Decode unsupported type from PostgreSQL</span></a></h2>
<p>When you retrieve some data from the <code v-pre>PostgreSQL</code> there are can be data types that we don't support yet.<br>
To deal with this situation, you can use <code v-pre>custom_decoders</code> parameter in <code v-pre>result()</code> and <code v-pre>as_class()</code> methods.</p>
<p>Let's assume we have table <code v-pre>for_test</code> in the database and <code v-pre>PSQLPy</code> doesn't support (only for demonstration) <code v-pre>VARCHAR</code> type:</p>
<table>
<thead>
<tr>
<th style="text-align:center">database type</th>
<th style="text-align:center">database column name</th>
</tr>
</thead>
<tbody>
<tr>
<td style="text-align:center">VARCHAR</td>
<td style="text-align:center">nickname</td>
</tr>
</tbody>
</table>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">from</span> typing <span class="token keyword">import</span> Final<span class="token punctuation">,</span> Any

<span class="token keyword">from</span> psqlpy <span class="token keyword">import</span> ConnectionPool<span class="token punctuation">,</span> QueryResult
<span class="token keyword">from</span> psqlpy<span class="token punctuation">.</span>extra_types <span class="token keyword">import</span> PyCustomType


<span class="token keyword">def</span> <span class="token function">nickname_decoder</span><span class="token punctuation">(</span>bytes_from_psql<span class="token punctuation">:</span> <span class="token builtin">bytes</span> <span class="token operator">|</span> <span class="token boolean">None</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token builtin">str</span><span class="token punctuation">:</span>
    <span class="token keyword">return</span> bytes_from_psql<span class="token punctuation">.</span>decode<span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token keyword">if</span> bytes_from_psql <span class="token keyword">else</span> <span class="token boolean">None</span>


<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token comment"># It uses default connection parameters</span>
    db_pool<span class="token punctuation">:</span> Final <span class="token operator">=</span> ConnectionPool<span class="token punctuation">(</span><span class="token punctuation">)</span>

    result<span class="token punctuation">:</span> QueryResult <span class="token operator">=</span> <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>execute<span class="token punctuation">(</span>
        <span class="token string">"SELECT * FROM for_test"</span><span class="token punctuation">,</span>
        <span class="token punctuation">[</span>PyCustomType<span class="token punctuation">(</span><span class="token string">b"SomeDataInBytes"</span><span class="token punctuation">)</span><span class="token punctuation">]</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>

    parsed_result<span class="token punctuation">:</span> <span class="token builtin">list</span><span class="token punctuation">[</span><span class="token builtin">dict</span><span class="token punctuation">[</span><span class="token builtin">str</span><span class="token punctuation">,</span> Any<span class="token punctuation">]</span><span class="token punctuation">]</span> <span class="token operator">=</span> result<span class="token punctuation">.</span>result<span class="token punctuation">(</span>
        custom_decoders<span class="token operator">=</span><span class="token punctuation">{</span>
            <span class="token string">"nickname"</span><span class="token punctuation">:</span> nickname_decoder<span class="token punctuation">,</span>
        <span class="token punctuation">}</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>
    db_pool<span class="token punctuation">.</span>close<span class="token punctuation">(</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><div class="hint-container important">
<p class="hint-container-title">Important</p>
<p>Rules about <code v-pre>custom_decoders</code> parameter:</p>
<ul>
<li>The key of the dict must be the name of the field on which you want to apply the decode function.</li>
<li>If you use aliases for the result field name, you must specify the alias.</li>
<li>The key of the dict must be in <strong>lowercase</strong>.</li>
</ul>
</div>
</div></template>


