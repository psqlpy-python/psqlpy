# create-codepen

A library creating codepen with api.

## loadCodePens

Load codepen through dom, this should be the same as the codepen embed script, while we are not calling `loadCodePen(".codepen")` directly.

```ts
export const loadCodePens: (selector = ".codepen") => void;
```

Example:

```html
<p
  class="code-pen-test"
  data-height="265"
  data-theme-id="light"
  data-default-tab="js,result"
  data-user="Mamboleoo"
  data-slug-hash="XWJPxpZ"
  style="
        height: 265px;
        box-sizing: border-box;
        display: flex;
        align-items: center;
        justify-content: center;
        border: 2px solid;
        margin: 1em 0;
        padding: 1em;
      "
  data-pen-title="Walkers - How to"
>
  <span
    >See the Pen
    <a href="https://codepen.io/Mamboleoo/pen/XWJPxpZ"> Walkers - How to</a>
    by Louis Hoebregts (<a href="https://codepen.io/Mamboleoo">@Mamboleoo</a>)
    on <a href="https://codepen.io">CodePen</a>.</span
  >
</p>
<script type="module">
  import { loadCodePens } from "https://unpkg.com/create-codepen";

  loadCodePens(".code-pen-tst");
</script>
```

## openCodePens

Open codepen in new window

```ts
export const openCodePens: (selector?: string) => void;
```

Example:

```html
<p
  class="code-pen-open"
  data-theme-id="light"
  data-default-tab="js,result"
  data-user="Mamboleoo"
  data-slug-hash="XWJPxpZ"
  data-pen-title="Walkers - How to"
>
  See the Pen Walkers - How to by Louis Hoebregts (@Mamboleoo) on CodePen.
</p>
<script type="module">
  import { loadCodePens } from "https://unpkg.com/create-codepen";

  loadCodePens(".code-pen-open");
</script>
```

## renderCodePen

Generate a codepen iframe through options.

If a valid selector is provided, the codepen will be rendered inside selector element. Otherwise it will be rendered in new window.

```ts
interface CodePenStyleOptions {
  /**
   * @default 300
   */
  height?: number | string;
  /**
   * @default none
   */
  border?: "none" | "thin" | "thick";
  /**
   * @default #000000
   */
  "border-color"?: string;
  /**
   * @default #3d3d3e
   */
  "tab-bar-color"?: string;
  /**
   * @default #76daff
   */
  "tab-link-color"?: string;
  /**
   * @default #cccccc
   */
  "active-tab-color"?: string;
  /**
   * @default #000000
   */
  "active-link-color"?: string;
  /**
   * @default #ffffff
   */
  "link-logo-color"?: string;
  /**
   * Additional class name
   */
  class?: string;
  "custom-css-url"?: string;
}
interface CodePenDomOptions
  extends CodePenStyleOptions,
    Record<string, unknown> {
  /**
   * Id of theme
   * @default 0
   */
  "theme-id"?: string | number;
  "slug-hash"?: string;
  user?: string;
  /**
   * @description one of or a set of "html" | "css" | "js" | "result"
   * @default "result"
   */
  "default-tab"?: string;
  animations?: "run" | "stop-after-5";
  preview?: "true" | "false";
  /**
   * @default 1
   */
  zoom?: 1 | 0.5 | 0.25;
  token?: string;
  "pen-title"?: string;
  /**
   * @deprecated use "slug-hash" instead
   */
  href?: string;
  /**
   * @deprecated use "animations" instead
   */
  safe?: "true";
  /**
   * @deprecated use "default-tab" instead
   */
  type?: string;
  /** @private */
  name?: string;
}
interface CodePenPrefillOptions {
  title?: string;
  description?: string;
  head?: string;
  tags?: string | string[];
  html_classes?: string | string[];
  stylesheets?: string | string[];
  scripts?: string | string[];
}
interface CodePenOptions
  extends Omit<CodePenDomOptions, "name" | "type" | "href" | "safe"> {
  /** @private */
  data?: string;
  prefill?: CodePenPrefillOptions;
  /**
   * @default "false"
   */
  editable?: "true" | "false";
}

export const renderCodePen: (
  options: CodePenOptions,
  selector?: string | HTMLElement,
) => void;
```

Example:

```html
<div class="api"></div>
<div class="prefill"></div>
<button class="codepen-api-open">Open</button>

<script type="module">
  import { renderCodePen } from "https://unpkg.com/create-codepen";

  // render codepen in class="api"
  renderCodePen(
    {
      "slug-hash": "XWJPxpZ",
      height: 265,
      "theme-id": "light",
      "default-tab": "js,result",
      user: "Mamboleoo",
      title: "Walkers - How to",
    },
    ".api",
  );

  // render codepen in class="prefill"
  renderCodePen(
    {
      "default-tab": "js,result",
      prefill: {
        title: "React Basics Demo",
        description:
          "Shows how to use React and React DOM to render a module with props onto the page",
        tags: ["react", "react-docs-demo"],
        html_classes: ["loading", "no-js"],
        head: '<meta name="viewport" content="width=device-width, initial-scale=1">',
        stylesheets: "https://unpkg.com/normalize.css@8.0.1/normalize.css",
        scripts: [
          "https://cdnjs.cloudflare.com/ajax/libs/react/16.6.3/umd/react.production.min.js",
          "https://cdnjs.cloudflare.com/ajax/libs/react-dom/16.6.3/umd/react-dom.production.min.js",
        ],
        html: `\
<div id="root"></div>
`,
        css: `\
$gray: #ccc;
body {
  background: $gray;
  margin: 0;
  padding: 1rem;
}
.module {
  background: white;
  padding: 1rem;
  border-radius: 4px;
  border: 1px solid #999;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.25);
  h1 {
    margin: 0 0 1rem 0;
  }
}
`,
        js: `\
class Welcome extends React.Component {
  render() {
    return <div class="module">
      <h1>
        Hello, {this.props.name}
      </h1>
      <p>It's a good day to build websites.</p>
    </div>;
  }
}
ReactDOM.render(
  <Welcome name="Chris"></Welcome>,
  document.getElementById('root')
);
`,
        css_pre_processor: "scss",
        js_pre_processor: "babel",
      },
    },
    ".prefill",
  );

  document.querySelector(".codepen-api-open").addEventListener("click", () => {
    renderCodePen({
      "default-tab": "js,result",
      prefill: {
        title: "React Basics Demo",
        description:
          "Shows how to use React and React DOM to render a module with props onto the page",
        tags: ["react", "react-docs-demo"],
        html_classes: ["loading", "no-js"],
        head: '<meta name="viewport" content="width=device-width, initial-scale=1">',
        stylesheets: "https://unpkg.com/normalize.css@8.0.1/normalize.css",
        scripts: [
          "https://cdnjs.cloudflare.com/ajax/libs/react/16.6.3/umd/react.production.min.js",
          "https://cdnjs.cloudflare.com/ajax/libs/react-dom/16.6.3/umd/react-dom.production.min.js",
        ],
        html: `\
<div id="root"></div>
`,
        css: `\
$gray: #ccc;
body {
  background: $gray;
  margin: 0;
  padding: 1rem;
}
.module {
  background: white;
  padding: 1rem;
  border-radius: 4px;
  border: 1px solid #999;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.25);
  h1 {
    margin: 0 0 1rem 0;
  }
}
`,
        js: `\
class Welcome extends React.Component {
  render() {
    return <div class="module">
      <h1>
        Hello, {this.props.name}
      </h1>
      <p>It's a good day to build websites.</p>
    </div>;
  }
}
ReactDOM.render(
  <Welcome name="Chris"></Welcome>,
  document.getElementById('root')
);
            `,
        css_pre_processor: "scss",
        js_pre_processor: "babel",
      },
    });
  });
</script>
```
