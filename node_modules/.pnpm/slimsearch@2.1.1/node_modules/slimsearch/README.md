# SlimSearch

[![npm](https://img.shields.io/npm/v/slimsearch?color=%23ff00dd)](https://www.npmjs.com/package/slimsearch)
[![npm downloads](https://img.shields.io/npm/dw/slimsearch)](https://www.npmjs.com/package/slimsearch)
[![types](https://img.shields.io/npm/types/slimsearch)](https://mister-hope.github.io/slimsearch/)

[![Test](https://github.com/Mister-Hope/slimsearch/actions/workflows/main.yml/badge.svg)](https://github.com/Mister-Hope/slimsearch/actions/workflows/main.yml)
[![codecov](https://codecov.io/gh/Mister-Hope/slimsearch/branch/main/graph/badge.svg?token=YQGZxImdqj)](https://codecov.io/gh/Mister-Hope/slimsearch)

`slimsearch` is a tiny but powerful in-memory full-text search engine written in
JavaScript. It is respectful of resources, and it can comfortably run both in
Node and in the browser.

## SlimSearch is based on [MiniSearch](https://lucaong.github.io/minisearch/), sharing the SAME index structure.

## Use case

`slimsearch` addresses use cases where full-text search features are needed
(e.g. prefix search, fuzzy search, ranking, boosting of fields…), but the data
to be indexed can fit locally in the process memory. While you won't index the
whole Internet with it, there are surprisingly many use cases that are served
well by `slimsearch`. By storing the index in local memory, `slimsearch` can
work offline, and can process queries quickly, without network latency.

A prominent use-case is real time search "as you type" in web and mobile
applications, where keeping the index on the client enables fast and reactive
UIs, removing the need to make requests to a search server.

## Features

- Memory-efficient index, designed to support memory-constrained use cases
  like mobile browsers.

- Exact match, prefix search, fuzzy match, field boosting.

- Auto-suggestion engine, for auto-completion of search queries.

- Modern search result ranking algorithm.

- Documents can be added and removed from the index at any time.

- Zero external dependencies.

`slimsearch` strives to expose a simple API that provides the building blocks to
build custom solutions, while keeping a small and well tested codebase.

## Installation

With `npm`:

```shell
npm install slimsearch
```

With `yarn`:

```shell
yarn add slimsearch
```

With `pnpm`:

```shell
pnpm add slimsearch
```

Then `require` or `import` it in your project:

```js
// If you are using import:
import {
  createIndex,
  // apis...
} from "slimsearch";

// If you are using require:
const {
  createIndex,
  // apis...
} = require("slimsearch");
```

## Usage

### Basic usage

```ts
import { addAll, createIndex, search } from "slimsearch";

// A collection of documents for our examples
const documents = [
  {
    id: 1,
    title: "Moby Dick",
    text: "Call me Ishmael. Some years ago...",
    category: "fiction",
  },
  {
    id: 2,
    title: "Zen and the Art of Motorcycle Maintenance",
    text: "I can see by my watch...",
    category: "fiction",
  },
  {
    id: 3,
    title: "Neuromancer",
    text: "The sky above the port was...",
    category: "fiction",
  },
  {
    id: 4,
    title: "Zen and the Art of Archery",
    text: "At first sight it must seem...",
    category: "non-fiction",
  },
  // ...and more
];

const index = createIndex({
  fields: ["title", "text"], // fields to index for full-text search
  storeFields: ["title", "category"], // fields to return with search results
});

// Index all documents
addAll(index, documents);

// Search with default options
const results = search(index, "zen art motorcycle");
// => [
//   { id: 2, title: 'Zen and the Art of Motorcycle Maintenance', category: 'fiction', score: 2.77258, match: { ... } },
//   { id: 4, title: 'Zen and the Art of Archery', category: 'non-fiction', score: 1.38629, match: { ... } }
// ]
```

### Search options

`slimsearch` supports several options for more advanced search behavior:

```js
import { addAll, createIndex, search } from "slimsearch";

// Search only specific fields
search(index, "zen", { fields: ["title"] });

// Boost some fields (here "title")
search(index, "zen", { boost: { title: 2 } });

// Prefix search (so that 'moto' will match 'motorcycle')
search(index, "moto", { prefix: true });

// Search within a specific category
search(index, "zen", {
  filter: (result) => result.category === "fiction",
});

// Fuzzy search, in this example, with a max edit distance of 0.2 * term length,
// rounded to nearest integer. The mispelled 'ismael' will match 'ishmael'.
search(index, "ismael", { fuzzy: 0.2 });

// You can set the default search options upon initialization
index = createIndex({
  fields: ["title", "text"],
  searchOptions: {
    boost: { title: 2 },
    fuzzy: 0.2,
  },
});
addAll(index, documents);

// It will now by default perform fuzzy search and boost "title":
search(index, "zen and motorcycles");
```

### Auto suggestions

`slimsearch` can suggest search queries given an incomplete query:

```ts
import { autoSuggest } from "slimsearch";

autoSuggest(index, "zen ar");
// => [ { suggestion: 'zen archery art', terms: [ 'zen', 'archery', 'art' ], score: 1.73332 },
//      { suggestion: 'zen art', terms: [ 'zen', 'art' ], score: 1.21313 } ]
```

The `autoSuggest` method takes the same options as the `search` method, so you
can get suggestions for misspelled words using fuzzy search:

```ts
autoSuggest(index, "neromancer", { fuzzy: 0.2 });
// => [ { suggestion: 'neuromancer', terms: [ 'neuromancer' ], score: 1.03998 } ]
```

Suggestions are ranked by the relevance of the documents that would be returned
by that search.

Sometimes, you might need to filter auto suggestions to, say, only a specific
category. You can do so by providing a `filter` option:

```ts
autoSuggest(index, "zen ar", {
  filter: (result) => result.category === "fiction",
});
// => [ { suggestion: 'zen art', terms: [ 'zen', 'art' ], score: 1.21313 } ]
```

### Field extraction

By default, documents are assumed to be plain key-value objects with field names
as keys and field values as simple values. In order to support custom field
extraction logic (for example for nested fields, or non-string field values that
need processing before tokenization), a custom field extractor function can be
passed as the `extractField` option:

```ts
import { createIndex } from "slimsearch";

// Assuming that our documents look like:
const documents = [
  {
    id: 1,
    title: "Moby Dick",
    author: { name: "Herman Melville" },
    pubDate: new Date(1851, 9, 18),
  },
  {
    id: 2,
    title: "Zen and the Art of Motorcycle Maintenance",
    author: { name: "Robert Pirsig" },
    pubDate: new Date(1974, 3, 1),
  },
  {
    id: 3,
    title: "Neuromancer",
    author: { name: "William Gibson" },
    pubDate: new Date(1984, 6, 1),
  },
  {
    id: 4,
    title: "Zen in the Art of Archery",
    author: { name: "Eugen Herrigel" },
    pubDate: new Date(1948, 0, 1),
  },
  // ...and more
];

// We can support nested fields (author.name) and date fields (pubDate) with a
// custom `extractField` function:

const index = createIndex({
  fields: ["title", "author.name", "pubYear"],
  extractField: (document, fieldName) => {
    // If field name is 'pubYear', extract just the year from 'pubDate'
    if (fieldName === "pubYear") {
      const pubDate = document["pubDate"];
      return pubDate && pubDate.getFullYear().toString();
    }

    // Access nested fields
    return fieldName.split(".").reduce((doc, key) => doc && doc[key], document);
  },
});
```

The default field extractor can be obtained by calling `getDefaultValue('extractField')`.

### Tokenization

By default, documents are tokenized by splitting on Unicode space or punctuation characters. The tokenization logic can be easily changed by passing a custom tokenizer function as the `tokenize` option:

```ts
import { createIndex } from "slimsearch";

// Tokenize splitting by hyphen
const index = createIndex({
  fields: ["title", "text"],
  tokenize: (string, _fieldName) => string.split("-"),
});
```

Upon search, the same tokenization is used by default, but it is possible to pass a `tokenize` search option in case a different search-time tokenization is necessary:

```ts
import { createIndex } from "slimsearch";

// Tokenize splitting by hyphen
const index = createIndex({
  fields: ["title", "text"],
  tokenize: (string) => string.split("-"), // indexing tokenizer
  searchOptions: {
    tokenize: (string) => string.split(/[\s-]+/), // search query tokenizer
  },
});
```

The default tokenizer can be obtained by calling `getDefaultValue('tokenize')`.

### Term processing

Terms are downcased by default. No stemming is performed, and no stop-word list is applied. To customize how the terms are processed upon indexing, for example to normalize them, filter them, or to apply stemming, the `processTerm` option can be used. The `processTerm` function should return the processed term as a string, or a falsy value if the term should be discarded:

```ts
import { createIndex } from "slimsearch";

const stopWords = new Set([
  "and",
  "or",
  "to",
  "in",
  "a",
  "the" /* ...and more */,
]);

// Perform custom term processing (here discarding stop words and downcasing)
const index = createIndex({
  fields: ["title", "text"],
  processTerm: (term, _fieldName) =>
    stopWords.has(term) ? null : term.toLowerCase(),
});
```

By default, the same processing is applied to search queries. In order to apply a different processing to search queries, supply a `processTerm` search option:

```js
import { createIndex } from "slimsearch";

const index = createIndex({
  fields: ["title", "text"],
  processTerm: (term) => (stopWords.has(term) ? null : term.toLowerCase()), // index term processing
  searchOptions: {
    processTerm: (term) => term.toLowerCase(), // search query processing
  },
});
```

The default term processor can be obtained by calling `getDefaultValue('processTerm')`.

### API Documentation

Refer to the [API documentation](https://mister-hope.github.io/slimsearch/) for details about configuration options and methods.
