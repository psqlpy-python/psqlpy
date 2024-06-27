<template><div><p>We love contributions. This guide is for all fellas who want to make psqlpy better together.</p>
<p>There are several rules for contributors:</p>
<ul>
<li>Please do not add malware.</li>
<li>Please make sure that your request solves the problem.</li>
</ul>
<p>If you struggle with something or feel frustrated, you either create an issue, create a <a href="https://github.com/qaspen-python/psqlpy/discussions" target="_blank" rel="noopener noreferrer">discussions<ExternalLinkIcon/></a>. page or publish a draft PR and ask your question in the description.</p>
<p>We have lots of tests in CI. But since CI runs from first-time contributors should be approved, you better test locally. It just takes less time to prepare PR for merging.</p>
<h2 id="setting-up-environment" tabindex="-1"><a class="header-anchor" href="#setting-up-environment"><span>Setting up environment</span></a></h2>
<p>Since it's rust-first project you need to install rust on your computer.
The best way to do it is check <a href="https://www.rust-lang.org/tools/install" target="_blank" rel="noopener noreferrer">official site<ExternalLinkIcon/></a>.</p>
<p>After you install rust, you must prepare you python environment.
One of the best ways is follow <a href="https://www.maturin.rs/installation" target="_blank" rel="noopener noreferrer">maturin offical documentation<ExternalLinkIcon/></a> but below you can find all necessary commands.</p>
<div class="language-bash line-numbers-mode" data-ext="sh" data-title="sh"><pre v-pre class="language-bash"><code><span class="token operator">></span> python3 <span class="token parameter variable">-m</span> venv .venv
<span class="token operator">></span> <span class="token builtin class-name">source</span> .venv/bin/activate
<span class="token operator">></span> pip <span class="token function">install</span> <span class="token parameter variable">-U</span> pip maturin
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><p>Then you need to build <code v-pre>PSQLPy</code> project.</p>
<div class="language-bash line-numbers-mode" data-ext="sh" data-title="sh"><pre v-pre class="language-bash"><code>maturin develop
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div></div></div><p>After this step project is built and installed in your python environment you created in previous step.</p>
<h2 id="linting-and-type-checking" tabindex="-1"><a class="header-anchor" href="#linting-and-type-checking"><span>Linting and type checking</span></a></h2>
<p>We have pre-commit configured with all our settings. We highly recommend you to install it as a git hook using pre-commit install command.</p>
<p>But even without installation, you can run all lints manually:</p>
<div class="language-bash line-numbers-mode" data-ext="sh" data-title="sh"><pre v-pre class="language-bash"><code>pre-commit run <span class="token parameter variable">-a</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div></div></div><h2 id="testing" tabindex="-1"><a class="header-anchor" href="#testing"><span>Testing</span></a></h2>
<p>You need to have <code v-pre>PostgreSQL</code> server somewhere to run <code v-pre>pytest</code>.</p>
<p>Default credentials for testing <code v-pre>PostgreSQL</code> and you can configure it with env:</p>
<ul>
<li>host: <code v-pre>localhost</code> (env: POSTGRES_HOST)</li>
<li>user: <code v-pre>postgres</code> (env: POSTGRES_USER)</li>
<li>password: <code v-pre>postgres</code> (env: POSTGRES_PASSWORD)</li>
<li>port: <code v-pre>5432</code> (env: POSTGRES_PORT)</li>
<li>dbname: <code v-pre>psqlpy_test</code> (env: POSTGRES_DBNAME)</li>
</ul>
<p>You can run pytest and it will do the thing.</p>
<div class="language-bash line-numbers-mode" data-ext="sh" data-title="sh"><pre v-pre class="language-bash"><code>pytest <span class="token builtin class-name">.</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div></div></div></div></template>


