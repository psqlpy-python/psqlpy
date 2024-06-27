<template><div><h2 id="installation" tabindex="-1"><a class="header-anchor" href="#installation"><span>Installation</span></a></h2>
<p>You can install psqlpy with pip, poetry or directly from git using pip:</p>
<Tabs id="6" :data='[{"id":"pip"},{"id":"poetry"},{"id":"git"}]'>
<template #title0="{ value, isActive }">pip</template>
<template #title1="{ value, isActive }">poetry</template>
<template #title2="{ value, isActive }">git</template>
<template #tab0="{ value, isActive }">
<div class="language-bash line-numbers-mode" data-ext="sh" data-title="sh"><pre v-pre class="language-bash"><code>pip <span class="token function">install</span> psqlpy
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div></div></div></template>
<template #tab1="{ value, isActive }">
<div class="language-bash line-numbers-mode" data-ext="sh" data-title="sh"><pre v-pre class="language-bash"><code>poetry <span class="token function">add</span> psqlpy
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div></div></div></template>
<template #tab2="{ value, isActive }">
<div class="language-bash line-numbers-mode" data-ext="sh" data-title="sh"><pre v-pre class="language-bash"><code>pip <span class="token function">install</span> git+https://github.com/qaspen-python/psqlpy
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div></div></div></template>
</Tabs>
<p>After installation you are ready to start querying!</p>
<h2 id="first-request-to-the-database" tabindex="-1"><a class="header-anchor" href="#first-request-to-the-database"><span>First request to the database</span></a></h2>
<p>There is a minimal example of what you need to do to send your first query and receive result.<br>
Let's assume that we have table <code v-pre>users</code>:</p>
<table>
<thead>
<tr>
<th style="text-align:center">id</th>
<th style="text-align:center">name</th>
<th style="text-align:center">username</th>
</tr>
</thead>
<tbody>
<tr>
<td style="text-align:center">1</td>
<td style="text-align:center">Aleksandr</td>
<td style="text-align:center">chandr-andr</td>
</tr>
<tr>
<td style="text-align:center">2</td>
<td style="text-align:center">Michail</td>
<td style="text-align:center">insani7y</td>
</tr>
</tbody>
</table>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">import</span> asyncio
<span class="token keyword">from</span> typing <span class="token keyword">import</span> Final

<span class="token keyword">from</span> psqlpy <span class="token keyword">import</span> ConnectionPool<span class="token punctuation">,</span> QueryResult


<span class="token keyword">async</span> <span class="token keyword">def</span> <span class="token function">main</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token operator">-</span><span class="token operator">></span> <span class="token boolean">None</span><span class="token punctuation">:</span>
    <span class="token comment"># It uses default connection parameters</span>
    db_pool<span class="token punctuation">:</span> Final <span class="token operator">=</span> ConnectionPool<span class="token punctuation">(</span><span class="token punctuation">)</span>

    results<span class="token punctuation">:</span> Final<span class="token punctuation">[</span>QueryResult<span class="token punctuation">]</span> <span class="token operator">=</span> <span class="token keyword">await</span> db_pool<span class="token punctuation">.</span>execute<span class="token punctuation">(</span>
        <span class="token string">"SELECT * FROM users WHERE id = $1"</span><span class="token punctuation">,</span>
        <span class="token punctuation">[</span><span class="token number">2</span><span class="token punctuation">]</span><span class="token punctuation">,</span>
    <span class="token punctuation">)</span>

    dict_results<span class="token punctuation">:</span> Final<span class="token punctuation">[</span><span class="token builtin">list</span><span class="token punctuation">[</span><span class="token builtin">dict</span><span class="token punctuation">[</span>Any<span class="token punctuation">,</span> Any<span class="token punctuation">]</span><span class="token punctuation">]</span><span class="token punctuation">]</span> <span class="token operator">=</span> results<span class="token punctuation">.</span>result<span class="token punctuation">(</span><span class="token punctuation">)</span>
    db<span class="token punctuation">.</span>close<span class="token punctuation">(</span><span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><div class="hint-container tip">
<p class="hint-container-title">Tips</p>
<p>You must call <code v-pre>close()</code> on database pool when you application is shutting down.</p>
</div>
<div class="hint-container caution">
<p class="hint-container-title">Caution</p>
<p>You must not use <code v-pre>ConnectionPool.execute</code> method in high-load production code!<br>
It pulls new connection from connection pull each call.<br>
Recommended way to make queries is executing them with <code v-pre>Connection</code>, <code v-pre>Transaction</code> or <code v-pre>Cursor</code>.</p>
</div>
</div></template>


