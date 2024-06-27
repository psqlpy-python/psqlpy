<template><div><p>We have a lot of exceptions for different situations.<br>
They are split into different subclasses for <code v-pre>ConnectionPool</code>, <code v-pre>Connection</code>, <code v-pre>Transaction</code> and <code v-pre>Cursor</code>.</p>
<p>You can use them to define code behavior based on exceptions.</p>
<h2 id="exceptions-dependency" tabindex="-1"><a class="header-anchor" href="#exceptions-dependency"><span>Exceptions dependency:</span></a></h2>
<Mermaid id="mermaid-9" code="eJydk89OhDAQxu/7FJxNuPgAJvLHROMmCKwX42EsFRtLS6YtWWJ8d6FsXFAaYHtr5vfNTGe+7pQGTSMGJULlN9c7rzupUTrJnh4jZA3FpA1A0RhRouf7N15/C6UQlGgmRSIlt7HNyi2qHEEo2CwLDSqJaxV9ND4SWvdlVvK5TNpn4Ibuoa6ZKNfWStpc9sQl2sPhPrK6bpZdWK/V7YHcFgUOKvU7Squ0NnCt1vuyTH9erl5trikVGMaLcxtusLu9s9IgTDfpFsRHSoweHmHRb2e7y60uJvtrs5lsIySgJXM8YkSFsqrYaEcOLJWcvwH5XAQzaGgtmVhOuTy78/eYm5uNZhrGDvsPhFyeTOYA7qgmH/NNTH7cTAuDj+eTn9ycALrKp0ZoVtGHbljj8j/vzJQm"></Mermaid><h2 id="exceptions" tabindex="-1"><a class="header-anchor" href="#exceptions"><span>Exceptions:</span></a></h2>
<h3 id="rustpsqldriverpybaseerror" tabindex="-1"><a class="header-anchor" href="#rustpsqldriverpybaseerror"><span>RustPSQLDriverPyBaseError</span></a></h3>
<p>Main bas exception for all errors.</p>
<h3 id="rusttopyvaluemappingerror" tabindex="-1"><a class="header-anchor" href="#rusttopyvaluemappingerror"><span>RustToPyValueMappingError</span></a></h3>
<p>Error if it is not possible to covert rust type to python.</p>
<p>You can get it if you database contains data type that it not supported by this library.</p>
<h3 id="pytorustvaluemappingerror" tabindex="-1"><a class="header-anchor" href="#pytorustvaluemappingerror"><span>PyToRustValueMappingError</span></a></h3>
<p>Error if it is not possible to covert python type to rust.</p>
<p>You can get this exception when executing queries with parameters. So, if there are no parameters for the query, don't handle this error.</p>
<h3 id="uuidvalueconverterror" tabindex="-1"><a class="header-anchor" href="#uuidvalueconverterror"><span>UUIDValueConvertError</span></a></h3>
<p>Error if it's impossible to convert py string UUID into rust UUID.</p>
<h3 id="macaddrconversionerror" tabindex="-1"><a class="header-anchor" href="#macaddrconversionerror"><span>MacAddrConversionError</span></a></h3>
<p>Error if cannot convert MacAddr string value to rust type.</p>
<h3 id="baseconnectionpoolerror" tabindex="-1"><a class="header-anchor" href="#baseconnectionpoolerror"><span>BaseConnectionPoolError</span></a></h3>
<p>Base error for all Connection Pool errors.</p>
<h4 id="connectionpoolbuilderror" tabindex="-1"><a class="header-anchor" href="#connectionpoolbuilderror"><span>ConnectionPoolBuildError</span></a></h4>
<p>Error for errors in building connection pool.</p>
<h4 id="connectionpoolconfigurationerror" tabindex="-1"><a class="header-anchor" href="#connectionpoolconfigurationerror"><span>ConnectionPoolConfigurationError</span></a></h4>
<p>Error in connection pool configuration.</p>
<h4 id="connectionpoolexecuteerror" tabindex="-1"><a class="header-anchor" href="#connectionpoolexecuteerror"><span>ConnectionPoolExecuteError</span></a></h4>
<p>Error in connection pool execution.</p>
<h3 id="baseconnectionerror" tabindex="-1"><a class="header-anchor" href="#baseconnectionerror"><span>BaseConnectionError</span></a></h3>
<p>Base error for Connection errors.</p>
<h4 id="connectionexecuteerror" tabindex="-1"><a class="header-anchor" href="#connectionexecuteerror"><span>ConnectionExecuteError</span></a></h4>
<p>Error in connection execution.</p>
<h3 id="basetransactionerror" tabindex="-1"><a class="header-anchor" href="#basetransactionerror"><span>BaseTransactionError</span></a></h3>
<p>Base error for all transaction errors.</p>
<h4 id="transactionbeginerror" tabindex="-1"><a class="header-anchor" href="#transactionbeginerror"><span>TransactionBeginError</span></a></h4>
<p>Error in transaction begin.</p>
<h4 id="transactioncommiterror" tabindex="-1"><a class="header-anchor" href="#transactioncommiterror"><span>TransactionCommitError</span></a></h4>
<p>Error in transaction commit.</p>
<h4 id="transactionrollbackerror" tabindex="-1"><a class="header-anchor" href="#transactionrollbackerror"><span>TransactionRollbackError</span></a></h4>
<p>Error in transaction rollback.</p>
<h4 id="transactionsavepointerror" tabindex="-1"><a class="header-anchor" href="#transactionsavepointerror"><span>TransactionSavepointError</span></a></h4>
<p>Error in transaction savepoint.</p>
<h4 id="transactionexecuteerror" tabindex="-1"><a class="header-anchor" href="#transactionexecuteerror"><span>TransactionExecuteError</span></a></h4>
<p>Error in transaction execution.</p>
<h3 id="basecursorerror" tabindex="-1"><a class="header-anchor" href="#basecursorerror"><span>BaseCursorError</span></a></h3>
<p>Base error for Cursor errors.</p>
<h4 id="cursorstarterror" tabindex="-1"><a class="header-anchor" href="#cursorstarterror"><span>CursorStartError</span></a></h4>
<p>Error in cursor declare.</p>
<h4 id="cursorcloseerror" tabindex="-1"><a class="header-anchor" href="#cursorcloseerror"><span>CursorCloseError</span></a></h4>
<p>Error in cursor close.</p>
<h4 id="cursorfetcherror" tabindex="-1"><a class="header-anchor" href="#cursorfetcherror"><span>CursorFetchError</span></a></h4>
<p>Error in cursor fetch (any fetch).</p>
</div></template>


