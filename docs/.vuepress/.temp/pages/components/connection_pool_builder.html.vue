<template><div><p>Component allows you to create <code v-pre>ConnectionPool</code> with chainable methods. It doesn't have any difference from classic python-way initialization.</p>
<p>Every method has robust docstring but you can check all available methods here.</p>
<div class="language-python line-numbers-mode" data-ext="py" data-title="py"><pre v-pre class="language-python"><code><span class="token keyword">from</span> psqlpy <span class="token keyword">import</span> ConnectionPoolBuilder


database_pool <span class="token operator">=</span> <span class="token punctuation">(</span>
    ConnectionPoolBuilder<span class="token punctuation">(</span><span class="token punctuation">)</span>
    <span class="token punctuation">.</span>max_pool_size<span class="token punctuation">(</span><span class="token number">10</span><span class="token punctuation">)</span>
    <span class="token punctuation">.</span>user<span class="token punctuation">(</span><span class="token string">"psqlpy"</span><span class="token punctuation">)</span>
    <span class="token punctuation">.</span>password<span class="token punctuation">(</span><span class="token string">"psqlpy"</span><span class="token punctuation">)</span>
    <span class="token punctuation">.</span>dbname<span class="token punctuation">(</span><span class="token string">"psqlpy"</span><span class="token punctuation">)</span>
    <span class="token punctuation">.</span>host<span class="token punctuation">(</span><span class="token string">"192.0.0.1"</span><span class="token punctuation">)</span>
    <span class="token punctuation">.</span>port<span class="token punctuation">(</span><span class="token number">5432</span><span class="token punctuation">)</span>
    <span class="token punctuation">.</span>build<span class="token punctuation">(</span><span class="token punctuation">)</span>
<span class="token punctuation">)</span>
</code></pre><div class="line-numbers" aria-hidden="true"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><h2 id="connectionpoolbuilder-methods" tabindex="-1"><a class="header-anchor" href="#connectionpoolbuilder-methods"><span>ConnectionPoolBuilder methods</span></a></h2>
<h3 id="build" tabindex="-1"><a class="header-anchor" href="#build"><span>build</span></a></h3>
<p>Create new <code v-pre>ConnectionPool</code> from <code v-pre>ConnectionPoolBuilder</code>.</p>
<h3 id="max-pool-size" tabindex="-1"><a class="header-anchor" href="#max-pool-size"><span>max_pool_size</span></a></h3>
<p>Set maximum connection pool size.</p>
<h3 id="conn-recycling-method" tabindex="-1"><a class="header-anchor" href="#conn-recycling-method"><span>conn_recycling_method</span></a></h3>
<p>Set connection recycling method.</p>
<p>Connection recycling method is how a connection is recycled.</p>
<h3 id="user" tabindex="-1"><a class="header-anchor" href="#user"><span>user</span></a></h3>
<p>Set username to <code v-pre>PostgreSQL</code>.</p>
<h3 id="password" tabindex="-1"><a class="header-anchor" href="#password"><span>password</span></a></h3>
<p>Set password for <code v-pre>PostgreSQL</code>.</p>
<h3 id="dbname" tabindex="-1"><a class="header-anchor" href="#dbname"><span>dbname</span></a></h3>
<p>Set database name for the <code v-pre>PostgreSQL</code>.</p>
<h3 id="options" tabindex="-1"><a class="header-anchor" href="#options"><span>options</span></a></h3>
<p>Set command line options used to configure the server.</p>
<h3 id="application-name" tabindex="-1"><a class="header-anchor" href="#application-name"><span>application_name</span></a></h3>
<p>Set the value of the <code v-pre>application_name</code> runtime parameter.</p>
<h3 id="ssl-mode" tabindex="-1"><a class="header-anchor" href="#ssl-mode"><span>ssl_mode</span></a></h3>
<p>Set the SSL configuration.</p>
<h3 id="host" tabindex="-1"><a class="header-anchor" href="#host"><span>host</span></a></h3>
<p>Add a host to the configuration.</p>
<p>Multiple hosts can be specified by calling this method multiple times,
and each will be tried in order.
On Unix systems, a host starting with a <code v-pre>/</code> is interpreted
as a path to a directory containing Unix domain sockets.
There must be either no hosts,
or the same number of hosts as hostaddrs.</p>
<h3 id="hostaddr" tabindex="-1"><a class="header-anchor" href="#hostaddr"><span>hostaddr</span></a></h3>
<p>Add a hostaddr to the configuration.</p>
<p>Multiple hostaddrs can be specified by calling
this method multiple times, and each will be tried in order.
There must be either no hostaddrs,
or the same number of hostaddrs as hosts.</p>
<h3 id="port" tabindex="-1"><a class="header-anchor" href="#port"><span>port</span></a></h3>
<p>Add a port to the configuration.</p>
<p>Multiple ports can be specified by calling this method multiple times.
There must either be no ports,
in which case the default of 5432 is used,
a single port, in which it is used for all hosts,
or the same number of ports as hosts.</p>
<h3 id="connect-timeout" tabindex="-1"><a class="header-anchor" href="#connect-timeout"><span>connect_timeout</span></a></h3>
<p>Set the timeout applied to socket-level connection attempts.</p>
<p>Note that hostnames can resolve to multiple IP addresses,
and this timeout will apply to each address of each
host separately. Defaults to no limit.</p>
<h3 id="tcp-user-timeout" tabindex="-1"><a class="header-anchor" href="#tcp-user-timeout"><span>tcp_user_timeout</span></a></h3>
<p>Set the TCP user timeout.</p>
<p>This is ignored for Unix domain socket connections.
It is only supported on systems where TCP_USER_TIMEOUT is available
and will default to the system default if omitted or set to 0;
on other systems, it has no effect.</p>
<h3 id="target-session-attrs" tabindex="-1"><a class="header-anchor" href="#target-session-attrs"><span>target_session_attrs</span></a></h3>
<p>Set the requirements of the session.</p>
<p>This can be used to connect to the primary server in a
clustered database rather than one of the read-only
secondary servers. Defaults to <code v-pre>Any</code>.</p>
<h3 id="load-balance-hosts" tabindex="-1"><a class="header-anchor" href="#load-balance-hosts"><span>load_balance_hosts</span></a></h3>
<p>Set the host load balancing behavior.</p>
<p>Defaults to <code v-pre>disable</code>.</p>
<h3 id="keepalives" tabindex="-1"><a class="header-anchor" href="#keepalives"><span>keepalives</span></a></h3>
<p>Control the use of TCP keepalive.</p>
<p>This is ignored for Unix domain socket connections.</p>
<p>Defaults to <code v-pre>true</code>.</p>
<h3 id="keepalives-idle" tabindex="-1"><a class="header-anchor" href="#keepalives-idle"><span>keepalives_idle</span></a></h3>
<p>Set the amount of idle time before a keepalive packet is sent on the connection.</p>
<p>This is ignored for Unix domain sockets,
or if the <code v-pre>keepalives</code> option is disabled.</p>
<p>Defaults to 2 hours.</p>
<h3 id="keepalives-interval" tabindex="-1"><a class="header-anchor" href="#keepalives-interval"><span>keepalives_interval</span></a></h3>
<p>Sets the time interval between TCP keepalive probes.</p>
<p>On Windows, this sets the value of the
tcp_keepalive struct keepalive interval field.</p>
<p>This is ignored for Unix domain sockets,
or if the <code v-pre>keepalives</code> option is disabled.</p>
<h3 id="keepalives-retries" tabindex="-1"><a class="header-anchor" href="#keepalives-retries"><span>keepalives_retries</span></a></h3>
<p>Sets the maximum number of TCP keepalive probes that will be sent before dropping a connection.</p>
<p>This is ignored for Unix domain sockets, or if the <code v-pre>keepalives</code> option is disabled.</p>
</div></template>


