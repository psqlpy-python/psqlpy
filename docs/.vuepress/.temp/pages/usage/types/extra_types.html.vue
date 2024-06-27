<template><div><p>PSQLPy has additional types due to the inability to accurately recognize the type passed from Python.</p>
<p>All extra types available from Python with mapping to PostgreSQL type and Rust type.</p>
<table>
<thead>
<tr>
<th style="text-align:center">PSQLPy type</th>
<th style="text-align:center">PostgreSQL type</th>
<th style="text-align:center">Rust Type</th>
</tr>
</thead>
<tbody>
<tr>
<td style="text-align:center">BigInt</td>
<td style="text-align:center">BigInt</td>
<td style="text-align:center">i64</td>
</tr>
<tr>
<td style="text-align:center">Integer</td>
<td style="text-align:center">Integer</td>
<td style="text-align:center">i32</td>
</tr>
<tr>
<td style="text-align:center">SmallInt</td>
<td style="text-align:center">SmallInt</td>
<td style="text-align:center">i16</td>
</tr>
<tr>
<td style="text-align:center">Float32</td>
<td style="text-align:center">FLOAT4</td>
<td style="text-align:center">f32</td>
</tr>
<tr>
<td style="text-align:center">Float64</td>
<td style="text-align:center">FLOAT8</td>
<td style="text-align:center">f64</td>
</tr>
<tr>
<td style="text-align:center">PyVarChar</td>
<td style="text-align:center">VarChar</td>
<td style="text-align:center">String</td>
</tr>
<tr>
<td style="text-align:center">PyText</td>
<td style="text-align:center">Text</td>
<td style="text-align:center">String</td>
</tr>
<tr>
<td style="text-align:center">PyJSON</td>
<td style="text-align:center">JSON</td>
<td style="text-align:center">serde::Value</td>
</tr>
<tr>
<td style="text-align:center">PyJSONB</td>
<td style="text-align:center">JSONB</td>
<td style="text-align:center">serde::Value</td>
</tr>
<tr>
<td style="text-align:center">PyMacAddr6</td>
<td style="text-align:center">MacAddr</td>
<td style="text-align:center">MacAddr6</td>
</tr>
<tr>
<td style="text-align:center">PyMacAddr8</td>
<td style="text-align:center">MacAddr8</td>
<td style="text-align:center">MacAddr8</td>
</tr>
</tbody>
</table>
<h2 id="bigint-integer-smallint-float32-float64" tabindex="-1"><a class="header-anchor" href="#bigint-integer-smallint-float32-float64"><span>BigInt &amp; Integer &amp; SmallInt &amp; Float32 &amp; Float64</span></a></h2>
<p>When integer is passed from Python to Rust, it's impossible to understand what type is required on the Database side.<br>
Because of this restriction if you are trying to insert or update number value, you need to specify type on Python side explicitly.</p>
<p>Let's assume we have table <code v-pre>numbers</code> in the database:</p>
<table>
<thead>
<tr>
<th style="text-align:center">database type</th>
<th style="text-align:center">database column name</th>
</tr>
</thead>
<tbody>
<tr>
<td style="text-align:center">SmallInt</td>
<td style="text-align:center">index</td>
</tr>
<tr>
<td style="text-align:center">Integer</td>
<td style="text-align:center">elf_life</td>
</tr>
<tr>
<td style="text-align:center">BigInt</td>
<td style="text-align:center">elon_musk_money</td>
</tr>
<tr>
<td style="text-align:center">FLOAT4</td>
<td style="text-align:center">rest_money</td>
</tr>
<tr>
<td style="text-align:center">FLOAT8</td>
<td style="text-align:center">company_money</td>
</tr>
</tbody>
</table>
<p>And we want to INSERT new data to this table:</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">from</span> typing <span class="token keyword">import</span> Final

<span class="token keyword">from</span> psqlpy <span class="token keyword">import</span> ConnectionPool<span class="token punctuation">,</span> QueryResult
<span class="token keyword">from</span> psqlpy<span class="token punctuation">.</span>extra_types <span class="token keyword">import</span> SmallInt<span class="token punctuation">,</span> Integer<span class="token punctuation">,</span> BigInt<span class="token punctuation">,</span> Float32<span class="token punctuation">,</span> Float64


<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token comment"># It uses default connection parameters</span>
    db_pool<span class="token punctuation">:</span> Final <span class="token operator">=</span> ConnectionPool<span class="token punctuation">(</span><span class="token punctuation">)</span>

    <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>execute<span class="token punctuation">(</span>
        <span class="token string">"INSERT INTO numbers (index, elf_life, elon_musk_money) VALUES ($1, $2, $3, $4, $5)"</span><span class="token punctuation">,</span>
        <span class="token punctuation">[</span>SmallInt<span class="token punctuation">(</span><span class="token number">101</span><span class="token punctuation">)</span><span class="token punctuation">,</span> Integer<span class="token punctuation">(</span><span class="token number">10500</span><span class="token punctuation">)</span><span class="token punctuation">,</span> BigInt<span class="token punctuation">(</span><span class="token number">300000000000</span><span class="token punctuation">)</span><span class="token punctuation">,</span> Float32<span class="token punctuation">(</span><span class="token number">123.11</span><span class="token punctuation">)</span><span class="token punctuation">,</span> Float64<span class="token punctuation">(</span><span class="token number">222.12</span><span class="token punctuation">)</span><span class="token punctuation">]</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>
    db_pool<span class="token punctuation">.</span>close<span class="token punctuation">(</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><div class="hint-container important">
<p class="hint-container-title">Important</p>
<p>These types are limited only by the upper bound.<br>
These classes work not only as wrappers, but also as validators.
For example, you can't pass integer bigger than 32,768 to SmallInt type.</p>
</div>
<h2 id="pyvarchar-pytext" tabindex="-1"><a class="header-anchor" href="#pyvarchar-pytext"><span>PyVarChar &amp; PyText</span></a></h2>
<p>When you need to pass string from Python to PSQLPy and this string must converted into Text PostgreSQL, you need to explicitly mark your string as <code v-pre>PyText</code>.<br>
If you don't work with PostgreSQL <code v-pre>TEXT</code> type, you can pass python <code v-pre>str</code> without any extra type.</p>
<p>Let's assume we have table <code v-pre>banners</code> in the database:</p>
<table>
<thead>
<tr>
<th style="text-align:center">database type</th>
<th style="text-align:center">database column name</th>
</tr>
</thead>
<tbody>
<tr>
<td style="text-align:center">VarChar</td>
<td style="text-align:center">title</td>
</tr>
<tr>
<td style="text-align:center">Text</td>
<td style="text-align:center">description</td>
</tr>
</tbody>
</table>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">from</span> typing <span class="token keyword">import</span> Final

<span class="token keyword">from</span> psqlpy <span class="token keyword">import</span> ConnectionPool<span class="token punctuation">,</span> QueryResult
<span class="token keyword">from</span> psqlpy<span class="token punctuation">.</span>extra_types <span class="token keyword">import</span> PyText


<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token comment"># It uses default connection parameters</span>
    db_pool<span class="token punctuation">:</span> Final <span class="token operator">=</span> ConnectionPool<span class="token punctuation">(</span><span class="token punctuation">)</span>

    <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>execute<span class="token punctuation">(</span>
        <span class="token string">"INSERT INTO banners (title, description) VALUES ($1, $2)"</span><span class="token punctuation">,</span>
        <span class="token punctuation">[</span><span class="token string">"SomeTitle"</span><span class="token punctuation">,</span> PyText<span class="token punctuation">(</span><span class="token string">"Very long description"</span><span class="token punctuation">)</span><span class="token punctuation">]</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>
    <span class="token comment"># Alternatively, you can do this:</span>
    <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>execute<span class="token punctuation">(</span>
        <span class="token string">"INSERT INTO banners (title, description) VALUES ($1, $2)"</span><span class="token punctuation">,</span>
        <span class="token punctuation">[</span>PyVarChar<span class="token punctuation">(</span><span class="token string">"SomeTitle"</span><span class="token punctuation">)</span><span class="token punctuation">,</span> PyText<span class="token punctuation">(</span><span class="token string">"Very long description"</span><span class="token punctuation">)</span><span class="token punctuation">]</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>
    db_pool<span class="token punctuation">.</span>close<span class="token punctuation">(</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h2 id="pyjson-pyjsonb" tabindex="-1"><a class="header-anchor" href="#pyjson-pyjsonb"><span>PyJSON &amp; PyJSONB</span></a></h2>
<p><code v-pre>PyJSON</code>/<code v-pre>PyJSONB</code> type exists only for situations when you want to set list of something to JSON/JSONB field.<br>
If you have default Python dict like above, you DON'T have to use <code v-pre>PyJSON</code>/<code v-pre>PyJSONB</code> type.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code>my_dict <span class="token operator">=</span> <span class="token punctuation">{</span>
    <span class="token string">"just"</span><span class="token punctuation">:</span> <span class="token string">"regular"</span><span class="token punctuation">,</span>
    <span class="token string">"python"</span><span class="token punctuation">:</span> <span class="token string">"dictionary"</span><span class="token punctuation">,</span>
    <span class="token string">"of"</span><span class="token punctuation">:</span> <span class="token punctuation">[</span>
        <span class="token string">"values"</span><span class="token punctuation">,</span>
    <span class="token punctuation">]</span><span class="token punctuation">,</span>
    <span class="token string">"with"</span><span class="token punctuation">:</span> <span class="token punctuation">{</span>
        <span class="token string">"nested"</span><span class="token punctuation">:</span> <span class="token string">"values"</span><span class="token punctuation">,</span>
    <span class="token punctuation">}</span>
<span class="token punctuation">}</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><p>On the other side, if you want to set list of values to JSON/JSONB field, you must wrap it in <code v-pre>PyJSON</code>/<code v-pre>PyJSONB</code> type, otherwise <code v-pre>PSQLPy</code> will assume that you passed an array (PostgreSQL <code v-pre>ARRAY</code>).</p>
<p>Let's assume we have table <code v-pre>users</code> in the database, and field <code v-pre>additional_user_info</code> can contain different type of data:</p>
<table>
<thead>
<tr>
<th style="text-align:center">database type</th>
<th style="text-align:center">database column name</th>
</tr>
</thead>
<tbody>
<tr>
<td style="text-align:center">JSONB</td>
<td style="text-align:center">additional_user_info</td>
</tr>
</tbody>
</table>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">from</span> typing <span class="token keyword">import</span> Final

<span class="token keyword">from</span> psqlpy <span class="token keyword">import</span> ConnectionPool<span class="token punctuation">,</span> QueryResult
<span class="token keyword">from</span> psqlpy<span class="token punctuation">.</span>extra_types <span class="token keyword">import</span> PyJSON


<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token comment"># It uses default connection parameters</span>
    db_pool<span class="token punctuation">:</span> Final <span class="token operator">=</span> ConnectionPool<span class="token punctuation">(</span><span class="token punctuation">)</span>
    list_for_jsonb_field <span class="token operator">=</span> <span class="token punctuation">[</span>
        <span class="token punctuation">{</span><span class="token string">"some"</span><span class="token punctuation">:</span> <span class="token string">"dict"</span><span class="token punctuation">}</span><span class="token punctuation">,</span>
        <span class="token punctuation">[</span>
            <span class="token punctuation">{</span><span class="token string">"nested"</span><span class="token punctuation">:</span> <span class="token string">"list of dicts"</span><span class="token punctuation">}</span><span class="token punctuation">,</span>
        <span class="token punctuation">]</span><span class="token punctuation">,</span>
    <span class="token punctuation">]</span>

    dict_for_jsonb_field <span class="token operator">=</span> <span class="token punctuation">{</span>
        <span class="token string">"regular"</span><span class="token punctuation">:</span> <span class="token string">"dict"</span><span class="token punctuation">,</span>
        <span class="token string">"with"</span><span class="token punctuation">:</span> <span class="token punctuation">[</span>
            <span class="token string">"list"</span><span class="token punctuation">,</span> <span class="token string">"of"</span><span class="token punctuation">,</span> <span class="token string">"values"</span><span class="token punctuation">,</span> <span class="token number">100</span><span class="token punctuation">,</span>
        <span class="token punctuation">]</span>
    <span class="token punctuation">}</span>

    <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>execute<span class="token punctuation">(</span>
        <span class="token string">"INSERT INTO users (additional_user_info) VALUES ($1)"</span><span class="token punctuation">,</span>
        <span class="token punctuation">[</span>PyJSONB<span class="token punctuation">(</span>list_for_jsonb_field<span class="token punctuation">)</span><span class="token punctuation">]</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>
    <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>execute<span class="token punctuation">(</span>
        <span class="token string">"INSERT INTO users (additional_user_info) VALUES ($1)"</span><span class="token punctuation">,</span>
        <span class="token punctuation">[</span>dict_for_jsonb_field<span class="token punctuation">,</span><span class="token punctuation">]</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>

    db_pool<span class="token punctuation">.</span>close<span class="token punctuation">(</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h2 id="pymacaddr6-pymacaddr8" tabindex="-1"><a class="header-anchor" href="#pymacaddr6-pymacaddr8"><span>PyMacAddr6 &amp; PyMacAddr8</span></a></h2>
<p>Mac addresses must be used with <code v-pre>PyMacAddr6</code> and <code v-pre>PyMacAddr8</code> types.</p>
<p>Let's assume we have table <code v-pre>devices</code> in the database:</p>
<table>
<thead>
<tr>
<th style="text-align:center">database type</th>
<th style="text-align:center">database column name</th>
</tr>
</thead>
<tbody>
<tr>
<td style="text-align:center">MACADDR</td>
<td style="text-align:center">device_macaddr6</td>
</tr>
<tr>
<td style="text-align:center">MACADDR8</td>
<td style="text-align:center">device_macaddr8</td>
</tr>
</tbody>
</table>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">from</span> typing <span class="token keyword">import</span> Final

<span class="token keyword">from</span> psqlpy <span class="token keyword">import</span> ConnectionPool<span class="token punctuation">,</span> QueryResult
<span class="token keyword">from</span> psqlpy<span class="token punctuation">.</span>extra_types <span class="token keyword">import</span> PyMacAddr6<span class="token punctuation">,</span> PyMacAddr8


<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token comment"># It uses default connection parameters</span>
    db_pool<span class="token punctuation">:</span> Final <span class="token operator">=</span> ConnectionPool<span class="token punctuation">(</span><span class="token punctuation">)</span>

    <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>execute<span class="token punctuation">(</span>
        <span class="token string">"INSERT INTO devices (device_macaddr6, device_macaddr8) VALUES ($1, $2)"</span><span class="token punctuation">,</span>
        <span class="token punctuation">[</span>
            PyMacAddr6<span class="token punctuation">(</span><span class="token string">"08:00:2b:01:02:03"</span><span class="token punctuation">)</span><span class="token punctuation">,</span>
            PyMacAddr8<span class="token punctuation">(</span><span class="token string">"08:00:2b:01:02:03:04:05"</span><span class="token punctuation">)</span><span class="token punctuation">,</span>
        <span class="token punctuation">]</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>

    db_pool<span class="token punctuation">.</span>close<span class="token punctuation">(</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div></div></template>


