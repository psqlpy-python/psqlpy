<template><div><p><code v-pre>PSQLPy</code> is a new Python driver for PostgreSQL fully written in Rust. It was inspired by <code v-pre>Psycopg3</code> and <code v-pre>AsyncPG</code>.</p>
<p>With <code v-pre>PSQLPy</code> you can:</p>
<ul>
<li>Make an interaction with the PostgeSQL in your application much faster (2-3 times).</li>
<li>Be sure that there won't be any unexpected errors.</li>
<li>Don't usually go to the documentation to search every question - we have awesome docstrings for every component.</li>
<li>Use <code v-pre>MyPy</code> (or any other Python type checker) with confidence that exactly the types specified in the typing will be returned.</li>
<li>Concentrate on writing your code, not understanding new abstractions in this library, we only have classes which represents PostgreSQL object (transaction, cursor, etc).</li>
</ul>
<div class="hint-container info">
<p class="hint-container-title">Info</p>
<p>It is extremely important to understand that the library will provide a noticeable acceleration in working with the database only if your queries are optimized.<br>
Otherwise, there will be acceleration, but not so significant</p>
</div>
<h2 id="important-notes" tabindex="-1"><a class="header-anchor" href="#important-notes"><span>Important notes</span></a></h2>
<p>All statements will be prepared by default. You can read more about it here <a href="https://www.postgresql.org/docs/current/sql-prepare.html" target="_blank" rel="noopener noreferrer">PostgreSQL Docs<ExternalLinkIcon/></a><br>
But in some situations this behavior can break you application. As an example, if you are using <code v-pre>PGBouncer</code> with <code v-pre>Transaction Pooling Mode</code> <a href="https://devcenter.heroku.com/articles/best-practices-pgbouncer-configuration#transaction-pooling-mode-recommended" target="_blank" rel="noopener noreferrer">Docs<ExternalLinkIcon/></a> or <code v-pre>Statement Pooling Mode</code> <a href="https://devcenter.heroku.com/articles/best-practices-pgbouncer-configuration#transaction-pooling-mode-recommended" target="_blank" rel="noopener noreferrer">Docs<ExternalLinkIcon/></a> you need to disable statement preparation. You can read how to do it in the next parts of the documentation.</p>
<h2 id="join-community" tabindex="-1"><a class="header-anchor" href="#join-community"><span>Join community!</span></a></h2>
<p>You can get support from the creators of <code v-pre>PSQLPy</code> in some social media:</p>
<ul>
<li><a href="https://t.me/+f3Y8mYKgXxhmYThi" target="_blank" rel="noopener noreferrer">Telegram<ExternalLinkIcon/></a></li>
</ul>
</div></template>


