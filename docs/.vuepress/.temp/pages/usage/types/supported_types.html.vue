<template><div><h2 id="simple-type" tabindex="-1"><a class="header-anchor" href="#simple-type"><span>Simple Type</span></a></h2>
<p>Here you can find all types supported by <code v-pre>PSQLPy</code>. If PSQLPy isn't <code v-pre>-</code>, you can go to the <code v-pre>Extra Types</code> for more information.</p>
<table>
<thead>
<tr>
<th style="text-align:center">Python type</th>
<th style="text-align:center">PSQLPy extra type</th>
<th style="text-align:center">PostgreSQL Type</th>
</tr>
</thead>
<tbody>
<tr>
<td style="text-align:center">None</td>
<td style="text-align:center">-</td>
<td style="text-align:center">NULL</td>
</tr>
<tr>
<td style="text-align:center">bool</td>
<td style="text-align:center">-</td>
<td style="text-align:center">BOOL</td>
</tr>
<tr>
<td style="text-align:center">bytes</td>
<td style="text-align:center">-</td>
<td style="text-align:center">BYTEA</td>
</tr>
<tr>
<td style="text-align:center">str</td>
<td style="text-align:center">-</td>
<td style="text-align:center">VARCHAR</td>
</tr>
<tr>
<td style="text-align:center">str</td>
<td style="text-align:center">PyVarChar</td>
<td style="text-align:center">VARCHAR</td>
</tr>
<tr>
<td style="text-align:center">str</td>
<td style="text-align:center">PyText</td>
<td style="text-align:center">TEXT</td>
</tr>
<tr>
<td style="text-align:center">str</td>
<td style="text-align:center">-</td>
<td style="text-align:center">XML</td>
</tr>
<tr>
<td style="text-align:center">int</td>
<td style="text-align:center">SmallInt</td>
<td style="text-align:center">SMALLINT</td>
</tr>
<tr>
<td style="text-align:center">int</td>
<td style="text-align:center">INTEGER</td>
<td style="text-align:center">INTEGER</td>
</tr>
<tr>
<td style="text-align:center">int</td>
<td style="text-align:center">-</td>
<td style="text-align:center">INTEGER</td>
</tr>
<tr>
<td style="text-align:center">int</td>
<td style="text-align:center">BIGINT</td>
<td style="text-align:center">BIGINT</td>
</tr>
<tr>
<td style="text-align:center">float</td>
<td style="text-align:center">-</td>
<td style="text-align:center">FLOAT4</td>
</tr>
<tr>
<td style="text-align:center">float</td>
<td style="text-align:center">Float32</td>
<td style="text-align:center">FLOAT4</td>
</tr>
<tr>
<td style="text-align:center">float</td>
<td style="text-align:center">Float64</td>
<td style="text-align:center">FLOAT8</td>
</tr>
<tr>
<td style="text-align:center">datetime.date</td>
<td style="text-align:center">-</td>
<td style="text-align:center">DATE</td>
</tr>
<tr>
<td style="text-align:center">datetime.time</td>
<td style="text-align:center">-</td>
<td style="text-align:center">TIME</td>
</tr>
<tr>
<td style="text-align:center">datetime.datetime</td>
<td style="text-align:center">-</td>
<td style="text-align:center">TIMESTAMP</td>
</tr>
<tr>
<td style="text-align:center">datetime.datetime</td>
<td style="text-align:center">-</td>
<td style="text-align:center">TIMESTAMPTZ</td>
</tr>
<tr>
<td style="text-align:center">UUID</td>
<td style="text-align:center">-</td>
<td style="text-align:center">UUID</td>
</tr>
<tr>
<td style="text-align:center">dict</td>
<td style="text-align:center">-</td>
<td style="text-align:center">JSONB</td>
</tr>
<tr>
<td style="text-align:center">dict</td>
<td style="text-align:center">PyJSONB</td>
<td style="text-align:center">JSONB</td>
</tr>
<tr>
<td style="text-align:center">dict</td>
<td style="text-align:center">PyJSON</td>
<td style="text-align:center">JSON</td>
</tr>
<tr>
<td style="text-align:center">Mac Address 6</td>
<td style="text-align:center">PyMacAddr6</td>
<td style="text-align:center">MacAddr</td>
</tr>
<tr>
<td style="text-align:center">Mac Address 8</td>
<td style="text-align:center">PyMacAddr8</td>
<td style="text-align:center">MacAddr</td>
</tr>
<tr>
<td style="text-align:center">IPv4Address</td>
<td style="text-align:center">-</td>
<td style="text-align:center">INET</td>
</tr>
<tr>
<td style="text-align:center">IPv6Address</td>
<td style="text-align:center">-</td>
<td style="text-align:center">INET</td>
</tr>
<tr>
<td style="text-align:center">decimal.Decimal</td>
<td style="text-align:center">-</td>
<td style="text-align:center">NUMERIC</td>
</tr>
<tr>
<td style="text-align:center">int/str</td>
<td style="text-align:center">Money</td>
<td style="text-align:center">MONEY</td>
</tr>
</tbody>
</table>
<div class="hint-container important">
<p class="hint-container-title">Important</p>
<p>DECIMAL PostgreSQL type isn't supported, use NUMERIC instead.</p>
</div>
<h2 id="array-type" tabindex="-1"><a class="header-anchor" href="#array-type"><span>Array Type</span></a></h2>
<p>You can make arrays with any type of <code v-pre>Simple Type</code>s.</p>
<h4 id="example" tabindex="-1"><a class="header-anchor" href="#example"><span>Example:</span></a></h4>
<div class="language-sql line-numbers-mode" data-ext="sql" data-title="sql"><pre v-pre class="language-sql"><code><span class="token keyword">CREATE</span> <span class="token keyword">TABLE</span> test <span class="token punctuation">(</span>arr_of_json JSONB ARRAY<span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div></div></div><h2 id="composite-type" tabindex="-1"><a class="header-anchor" href="#composite-type"><span>Composite Type</span></a></h2>
<p><code v-pre>PSQLPy</code> supports composite types.</p>
<p>You can create your own types in PostgreSQL, we will return you <code v-pre>dict</code>.</p>
<div class="language-sql line-numbers-mode" data-ext="sql" data-title="sql"><pre v-pre class="language-sql"><code><span class="token keyword">CREATE</span> <span class="token keyword">TYPE</span> custom_type <span class="token keyword">AS</span> <span class="token punctuation">(</span>name <span class="token keyword">VARCHAR</span><span class="token punctuation">,</span> metadata JSONB<span class="token punctuation">)</span><span class="token punctuation">;</span>
<span class="token keyword">CREATE</span> <span class="token keyword">TABLE</span> custom_table <span class="token punctuation">(</span>user_info custom_type<span class="token punctuation">)</span><span class="token punctuation">;</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div></div></div><p>Let's insert some data.</p>
<div class="language-sql line-numbers-mode" data-ext="sql" data-title="sql"><pre v-pre class="language-sql"><code><span class="token keyword">INSERT</span> <span class="token keyword">INTO</span> custom_table <span class="token keyword">VALUES</span> <span class="token punctuation">(</span><span class="token keyword">ROW</span><span class="token punctuation">(</span><span class="token string">'Alex'</span><span class="token punctuation">,</span> <span class="token string">'{"age": 50}'</span><span class="token punctuation">)</span><span class="token punctuation">)</span><span class="token punctuation">;</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div></div></div><p>Now we can see what result will be returned.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">from</span> typing <span class="token keyword">import</span> Final

<span class="token keyword">from</span> psqlpy <span class="token keyword">import</span> ConnectionPool<span class="token punctuation">,</span> QueryResult
<span class="token keyword">from</span> psqlpy<span class="token punctuation">.</span>extra_types <span class="token keyword">import</span> SmallInt<span class="token punctuation">,</span> Integer<span class="token punctuation">,</span> BigInt


<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token comment"># It uses default connection parameters</span>
    db_pool<span class="token punctuation">:</span> Final <span class="token operator">=</span> ConnectionPool<span class="token punctuation">(</span><span class="token punctuation">)</span>

    result <span class="token operator">=</span> <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>execute<span class="token punctuation">(</span>
        <span class="token string">"SELECT user_info FROM custom_table"</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>
    <span class="token keyword">print</span><span class="token punctuation">(</span>result<span class="token punctuation">.</span>result<span class="token punctuation">(</span><span class="token punctuation">)</span><span class="token punctuation">[</span><span class="token number">0</span><span class="token punctuation">]</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><p>It will return:</p>
<div class="language-json line-numbers-mode" data-ext="json" data-title="json"><pre v-pre class="language-json"><code><span class="token punctuation">[</span>
    <span class="token punctuation">{</span>
        <span class="token property">"user_info"</span><span class="token operator">:</span> <span class="token punctuation">{</span>
            <span class="token property">"name"</span><span class="token operator">:</span> <span class="token string">"Alex"</span><span class="token punctuation">,</span>
            <span class="token property">"metadata"</span><span class="token operator">:</span> <span class="token punctuation">{</span>
                <span class="token property">"age"</span><span class="token operator">:</span> <span class="token number">50</span><span class="token punctuation">,</span>
            <span class="token punctuation">}</span><span class="token punctuation">,</span>
        <span class="token punctuation">}</span><span class="token punctuation">,</span>
    <span class="token punctuation">}</span>
<span class="token punctuation">]</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h2 id="enum-type" tabindex="-1"><a class="header-anchor" href="#enum-type"><span>Enum Type</span></a></h2>
<p>You can use ENUM type in <code v-pre>PostgreSQL</code> and <code v-pre>Python</code>.</p>
<p>Let's assume we create <code v-pre>Enum</code> <code v-pre>Weather</code> and table with it.</p>
<div class="language-sql line-numbers-mode" data-ext="sql" data-title="sql"><pre v-pre class="language-sql"><code><span class="token keyword">CREATE</span> <span class="token keyword">TYPE</span> weather <span class="token keyword">AS</span> <span class="token keyword">ENUM</span> <span class="token punctuation">(</span><span class="token string">'sun'</span><span class="token punctuation">,</span> <span class="token string">'not sun'</span><span class="token punctuation">)</span>
<span class="token keyword">CREATE</span> <span class="token keyword">table</span> weather_plus <span class="token punctuation">(</span>is_weather_good weather<span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div></div></div><p>Let's see how we can INSERT and SELECT such data.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">from</span> enum <span class="token keyword">import</span> Enum
<span class="token keyword">from</span> typing <span class="token keyword">import</span> Final

<span class="token keyword">from</span> psqlpy <span class="token keyword">import</span> ConnectionPool<span class="token punctuation">,</span> QueryResult


<span class="token keyword">class</span> <span class="token class-name">Weather</span><span class="token punctuation">(</span><span class="token builtin">str</span><span class="token punctuation">,</span> Enum<span class="token punctuation">)</span><span class="token punctuation">:</span>
    SUN <span class="token operator">=</span> <span class="token string">"sun"</span>
    NOT_SUN <span class="token operator">=</span> <span class="token string">"not sun"</span>


<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token comment"># It uses default connection parameters</span>
    db_pool<span class="token punctuation">:</span> Final <span class="token operator">=</span> ConnectionPool<span class="token punctuation">(</span><span class="token punctuation">)</span>

    <span class="token comment"># Insert new data</span>
    <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>execute<span class="token punctuation">(</span>
        querystring<span class="token operator">=</span><span class="token string">"INSERT INTO weather_plus VALUES($1)"</span><span class="token punctuation">,</span>
        parameters<span class="token operator">=</span><span class="token punctuation">[</span>Weather<span class="token punctuation">.</span>SUN<span class="token punctuation">]</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>

    <span class="token comment"># Or you can pass string directly</span>
    <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>execute<span class="token punctuation">(</span>
        querystring<span class="token operator">=</span><span class="token string">"INSERT INTO weather_plus VALUES($1)"</span><span class="token punctuation">,</span>
        parameters<span class="token operator">=</span><span class="token punctuation">[</span><span class="token string">"sun"</span><span class="token punctuation">]</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>

    result <span class="token operator">=</span> <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>execute<span class="token punctuation">(</span>
        querystring<span class="token operator">=</span><span class="token string">"SELECT * FROM weather_plus"</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>
    <span class="token keyword">print</span><span class="token punctuation">(</span>result<span class="token punctuation">.</span>result<span class="token punctuation">(</span><span class="token punctuation">)</span><span class="token punctuation">[</span><span class="token number">0</span><span class="token punctuation">]</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><p>You will receive:</p>
<div class="language-json line-numbers-mode" data-ext="json" data-title="json"><pre v-pre class="language-json"><code><span class="token punctuation">[</span>
    <span class="token punctuation">{</span>
        <span class="token property">"is_weather_good"</span><span class="token operator">:</span> <span class="token string">"sun"</span><span class="token punctuation">,</span>
    <span class="token punctuation">}</span><span class="token punctuation">,</span>
<span class="token punctuation">]</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div></div></template>


